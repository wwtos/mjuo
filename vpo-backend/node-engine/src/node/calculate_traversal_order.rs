use std::collections::{BTreeMap, HashMap};
use std::ops::Range;

use crate::connection::{Connection, Primitive, Socket, SocketType};
use crate::errors::{NodeError, NodeWarning};
use crate::graph_manager::{GraphIndex, GraphManager};
use crate::node::{NodeIndex, NodeInitParams, NodeRow, NodeRuntime};
use crate::node_graph::{NodeConnectionData, NodeGraph};
use crate::nodes::{new_variant, NodeVariant};
use crate::resources::{ResourceTypeAndIndex, Resources};

use common::resource_manager::ResourceId;
use ddgg::VertexIndex;
use petgraph::algo::{greedy_feedback_arc_set, toposort};
use petgraph::prelude::*;
use rhai::Engine;
use smallvec::SmallVec;
use sound_engine::SoundConfig;

pub fn calculate_graph_traverse_order(original_graph: &NodeGraph) -> Vec<NodeIndex> {
    // traverse backward and build a traversal order

    let mut graph = StableGraph::<NodeIndex, NodeConnectionData>::new();
    let mut graph_lookup: HashMap<VertexIndex, petgraph::stable_graph::NodeIndex> = HashMap::new();

    for original_node_index in original_graph.node_indexes() {
        graph_lookup.insert(original_node_index.0, graph.add_node(original_node_index));
    }

    for original_edge_index in original_graph.edge_indexes() {
        let edge = original_graph.get_graph().get_edge(original_edge_index.0).unwrap();

        graph.add_edge(
            graph_lookup[&edge.get_from()],
            graph_lookup[&edge.get_to()],
            edge.data().clone(),
        );
    }

    let edges = greedy_feedback_arc_set(&graph);

    let mut connections: Vec<Connection> = Vec::new();
    let mut edge_indexes: Vec<EdgeIndex> = Vec::new();

    for edge in edges {
        let weight = edge.weight();

        connections.push(Connection {
            from_node: graph[edge.source()],
            to_node: graph[edge.target()],
            data: NodeConnectionData {
                from_socket: weight.from_socket.clone(),
                to_socket: weight.to_socket.clone(),
            },
        });

        edge_indexes.push(edge.id());
    }

    for edge in edge_indexes {
        graph.remove_edge(edge);
    }

    let node_order = toposort(&graph, None).unwrap();

    node_order
        .iter()
        .map(|index| *graph.node_weight(*index).unwrap())
        .collect::<Vec<crate::node::NodeIndex>>()
}

#[derive(Debug)]
pub struct NodeIoCount {
    pub node: NodeVariant,
    pub to_input: Vec<(usize, Vec<Primitive>)>,
    pub midi_inputs: Vec<usize>,
    pub value_inputs: Vec<usize>,
    pub stream_inputs: Vec<usize>,
    pub midi_outputs: Vec<Socket>,
    pub value_outputs: Vec<Socket>,
    pub stream_outputs: Vec<Socket>,
    pub resources_count: usize,
    pub resources_index: usize,
    pub midi_index: usize,
    pub value_index: usize,
    pub stream_index: usize,
    pub values_to_input: SmallVec<[(usize, Primitive); 4]>,
    pub socket_lookup: BTreeMap<Socket, usize>,
}

#[derive(Debug)]
pub struct IoSpec {
    pub nodes: BTreeMap<NodeIndex, NodeIoCount>,
    pub resources_tracking: Vec<(ResourceId, Option<ResourceTypeAndIndex>)>,
    pub nodes_linked_to_ui: Vec<(usize, NodeIndex)>,
    pub traversal_order: Vec<NodeIndex>,
}

#[derive(Debug, Clone)]
pub struct NodeMappedIo {
    pub stream_in: Range<usize>,
    pub midi_in: Range<usize>,
    pub value_in: Range<usize>,
    pub stream_out: Range<usize>,
    pub midi_out: Range<usize>,
    pub value_out: Range<usize>,
    pub resources: Range<usize>,
}

#[derive(Debug)]
pub struct Indexes {
    pub streams: Vec<Option<Range<usize>>>,
    pub midis: Vec<Option<Range<usize>>>,
    pub values: Vec<Option<Range<usize>>>,
    pub max_stream_channels: usize,
    pub max_midi_channels: usize,
    pub max_value_channels: usize,
    pub stream_count: usize,
    pub midi_count: usize,
    pub value_count: usize,
    pub node_io: BTreeMap<NodeIndex, NodeMappedIo>,
    pub resources_tracking: Vec<(ResourceId, Option<ResourceTypeAndIndex>)>,
}

pub fn calc_io_spec(
    graph: &NodeGraph,
    mut old_nodes: BTreeMap<NodeIndex, NodeVariant>,
    sound_config: &SoundConfig,
    script_engine: &Engine,
    resources: &Resources,
    current_time: i64,
    graph_manager: &GraphManager,
    default_channel_count: usize,
) -> Result<IoSpec, NodeError> {
    let traversal_order = calculate_graph_traverse_order(&graph);

    let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
    let mut warnings: Vec<(NodeIndex, NodeWarning)> = vec![];

    let mut resources_tracking: Vec<(ResourceId, Option<ResourceTypeAndIndex>)> = vec![];
    let mut nodes_linked_to_ui: Vec<(usize, NodeIndex)> = vec![];

    let mut nodes: BTreeMap<NodeIndex, NodeIoCount> = BTreeMap::new();

    let mut stream_i: usize = 0;
    let mut midi_i: usize = 0;
    let mut value_i: usize = 0;

    // now for the fun part ;)
    //
    // # Step 1, denormalize all of the nodes
    // Each type of data stream is put in a different array, lined up back to back
    // according to `traversel_order`
    for (vec_index, node_index) in traversal_order.iter().enumerate() {
        // create and init the node
        let node_instance = graph.get_node(*node_index).expect("node to exist");

        // pull out the old instance, if it exists
        let mut variant = if let Some(previous_node) = old_nodes.remove(node_index) {
            previous_node
        } else {
            new_variant(&node_instance.get_node_type(), &sound_config)?
        };

        // get the child graph info, if any
        let child_graph_info = node_instance.get_child_graph();

        let init_result_res = variant.init(NodeInitParams {
            props: node_instance.get_properties(),
            script_engine,
            resources,
            current_time,
            graph_manager,
            sound_config: &sound_config,
            node_state: node_instance.get_state(),
            child_graph: child_graph_info.clone(),
            default_channel_count,
        });

        // handle any errors from initializing the node
        let needed_resources = match init_result_res {
            Ok(init_result) => {
                for warning in init_result.warnings.into_iter() {
                    warnings.push((*node_index, warning))
                }

                init_result.value.needed_resources
            }
            Err(err) => {
                errors.push((*node_index, err));

                vec![]
            }
        };

        let resources_i = resources_tracking.len();
        for needed_resource in &needed_resources {
            let resource_index = resources.get_resource_index(needed_resource);

            resources_tracking.push((needed_resource.clone(), resource_index));
        }

        if variant.has_state() {
            nodes_linked_to_ui.push((vec_index, *node_index));
        }

        // count up how many inputs (and of which type) this node has
        let mut stream_inputs: Vec<usize> = vec![];
        let mut midi_inputs: Vec<usize> = vec![];
        let mut value_inputs: Vec<usize> = vec![];

        let mut to_input: Vec<(usize, Vec<Primitive>)> = vec![];
        let mut values_to_input = SmallVec::new();
        let mut socket_lookup = BTreeMap::new();

        // go through the node by all its inputs
        for socket in node_instance.list_input_sockets() {
            let default_row = node_instance.get_default(socket).unwrap();

            if let NodeRow::Input(socket, default) = default_row {
                match socket.socket_type() {
                    SocketType::Stream => {
                        socket_lookup.insert(socket.clone(), stream_inputs.len());
                        stream_inputs.push(socket.channels());
                    }
                    SocketType::Midi => {
                        socket_lookup.insert(socket.clone(), midi_inputs.len());
                        midi_inputs.push(socket.channels());
                    }
                    SocketType::Value => {
                        values_to_input.push((value_inputs.len(), default.clone().as_value().unwrap()));
                        socket_lookup.insert(socket.clone(), value_inputs.len());
                        value_inputs.push(socket.channels());

                        // if it's not receiving from anything, be sure to input its default
                        let is_connected = graph.get_input_connection_index(*node_index, &socket)?.is_some();
                        if !is_connected {
                            to_input.push((value_inputs.len() - 1, vec![default.as_value().unwrap()]));
                        }
                    }
                    _ => {}
                }
            } else {
                unreachable!();
            }
        }

        // create a list of all of the outputs
        let mut stream_output_sockets: Vec<Socket> = vec![];
        let mut midi_output_sockets: Vec<Socket> = vec![];
        let mut value_output_sockets: Vec<Socket> = vec![];

        for socket in node_instance.list_output_sockets() {
            match socket.socket_type() {
                SocketType::Stream => stream_output_sockets.push(socket.clone()),
                SocketType::Midi => midi_output_sockets.push(socket.clone()),
                SocketType::Value => value_output_sockets.push(socket.clone()),
                _ => {}
            }
        }

        nodes.insert(
            *node_index,
            NodeIoCount {
                node: variant,
                to_input,
                midi_inputs,
                value_inputs,
                stream_inputs,
                midi_outputs: midi_output_sockets.clone(),
                value_outputs: value_output_sockets.clone(),
                stream_outputs: stream_output_sockets.clone(),
                midi_index: midi_i,
                value_index: value_i,
                stream_index: stream_i,
                resources_count: needed_resources.len(),
                resources_index: resources_i,
                socket_lookup,
                values_to_input,
            },
        );

        midi_i += midi_output_sockets.iter().map(|x| x.channels()).sum::<usize>();
        value_i += value_output_sockets.iter().map(|x| x.channels()).sum::<usize>();
        stream_i += stream_output_sockets.iter().map(|x| x.channels()).sum::<usize>();
    }

    Ok(IoSpec {
        nodes,
        resources_tracking,
        nodes_linked_to_ui,
        traversal_order,
    })
}

pub fn calc_indexes(
    io_needed: &IoSpec,
    graph_index: GraphIndex,
    graph_manager: &GraphManager,
) -> Result<Indexes, NodeError> {
    let IoSpec {
        nodes, traversal_order, ..
    } = io_needed;

    let graph = graph_manager.get_graph(graph_index)?;

    // figure out how big our io array needs to be
    let stream_count = nodes
        .iter()
        .map(|(_, node)| node.stream_outputs.iter().map(|x| x.channels()).sum::<usize>())
        .sum();
    let midi_count = nodes
        .iter()
        .map(|(_, node)| node.midi_outputs.iter().map(|x| x.channels()).sum::<usize>())
        .sum();
    let value_count = nodes
        .iter()
        .map(|(_, node)| node.value_outputs.iter().map(|x| x.channels()).sum::<usize>())
        .sum();

    // figure out the max number of channels to create a default input
    let max_stream_channels = nodes
        .iter()
        .map(|(_, node)| node.stream_outputs.iter().map(Socket::channels).max().unwrap_or(1))
        .max()
        .unwrap_or(1);
    let max_midi_channels = nodes
        .iter()
        .map(|(_, node)| node.midi_outputs.iter().map(Socket::channels).max().unwrap_or(1))
        .max()
        .unwrap_or(1);
    let max_value_channels = nodes
        .iter()
        .map(|(_, node)| node.value_outputs.iter().map(Socket::channels).max().unwrap_or(1))
        .max()
        .unwrap_or(1);

    let mut stream_io: Vec<Option<Range<usize>>> = vec![];
    let mut midi_io: Vec<Option<Range<usize>>> = vec![];
    let mut value_io: Vec<Option<Range<usize>>> = vec![];
    let mut node_mapped_io: BTreeMap<NodeIndex, NodeMappedIo> = BTreeMap::new();

    // # Step 2, populate mappings between nodes
    // Now we know where all the nodes are, so we can tell each node where its inputs are
    for index in traversal_order {
        let instance = graph.get_node(*index).expect("node to exist");
        let io_setup = &nodes[index];

        let input_sockets = instance.list_input_sockets();
        let output_sockets = instance.list_output_sockets();

        let stream_io_input_index = stream_io.len();
        let midi_io_input_index = midi_io.len();
        let value_io_input_index = value_io.len();

        let mut stream_io_inputs = 0;
        let mut midi_io_inputs = 0;
        let mut value_io_inputs = 0;

        // let's look through this node's inputs
        for input in &input_sockets {
            // is this node's input socket connected to anything?
            if let Some(connection_index) = graph.get_input_connection_index(*index, input).unwrap() {
                // get the node that it's connected from
                let connection = graph.get_graph().get_edge(connection_index.0).expect("edge to exist");
                let from_index = NodeIndex(connection.get_from());

                // make sure it's not being connected to itself
                assert_ne!(connection.get_from(), connection.get_to());

                // ensure same channel length
                assert_eq!(
                    connection.data().from_socket.channels(),
                    connection.data().to_socket.channels()
                );

                // where is the other nodes' output location?
                let io_setup_of_other = &nodes[&from_index];

                // add it to the io mapping
                match input.socket_type() {
                    SocketType::Stream => {
                        let other_stream_pos = io_setup_of_other
                            .stream_outputs
                            .iter()
                            .position(|other_socket| other_socket == &connection.data().from_socket)
                            .unwrap()
                            + io_setup_of_other.stream_index;

                        stream_io.push(Some(other_stream_pos..(other_stream_pos + input.channels())));
                        stream_io_inputs += 1;
                    }
                    SocketType::Midi => {
                        let other_midi_pos = io_setup_of_other
                            .midi_outputs
                            .iter()
                            .position(|other_socket| other_socket == &connection.data().from_socket)
                            .unwrap()
                            + io_setup_of_other.midi_index;

                        midi_io.push(Some(other_midi_pos..(other_midi_pos + input.channels())));
                        midi_io_inputs += 1;
                    }
                    SocketType::Value => {
                        let other_value_pos = io_setup_of_other
                            .value_outputs
                            .iter()
                            .position(|other_socket| other_socket == &connection.data().from_socket)
                            .unwrap()
                            + io_setup_of_other.value_index;

                        value_io.push(Some(other_value_pos..(other_value_pos + input.channels())));
                        value_io_inputs += 1;
                    }
                    SocketType::NodeRef => {}
                }
            } else {
                // it's not connected to anything, so push in to nothing (to preserve alignment)
                match input.socket_type() {
                    SocketType::Stream => {
                        stream_io.push(None);
                        stream_io_inputs += 1;
                    }
                    SocketType::Midi => {
                        midi_io.push(None);
                        midi_io_inputs += 1;
                    }
                    SocketType::Value => {
                        value_io.push(None);
                        value_io_inputs += 1;
                    }
                    SocketType::NodeRef => {}
                }
            }
        }

        let stream_io_output_index = stream_io.len();
        let midi_io_output_index = midi_io.len();
        let value_io_output_index = value_io.len();

        let mut stream_io_outputs = 0;
        let mut midi_io_outputs = 0;
        let mut value_io_outputs = 0;

        for output in &output_sockets {
            // add outputs to the mapping
            match output.socket_type() {
                SocketType::Stream => {
                    let position_in_stream = io_setup
                        .stream_outputs
                        .iter()
                        .position(|other_socket| other_socket == *output)
                        .unwrap()
                        + io_setup.stream_index;

                    stream_io.push(Some(position_in_stream..(position_in_stream + output.channels())));
                    stream_io_outputs += 1;
                }
                SocketType::Midi => {
                    let position_in_midi = io_setup
                        .midi_outputs
                        .iter()
                        .position(|other_socket| other_socket == *output)
                        .unwrap()
                        + io_setup.midi_index;

                    midi_io.push(Some(position_in_midi..(position_in_midi + output.channels())));
                    midi_io_outputs += 1;
                }
                SocketType::Value => {
                    let position_in_value = io_setup
                        .value_outputs
                        .iter()
                        .position(|other_socket| other_socket == *output)
                        .unwrap()
                        + io_setup.value_index;

                    value_io.push(Some(position_in_value..(position_in_value + output.channels())));
                    value_io_outputs += 1;
                }
                SocketType::NodeRef => {}
            }
        }

        node_mapped_io.insert(
            *index,
            NodeMappedIo {
                stream_in: stream_io_input_index..(stream_io_input_index + stream_io_inputs),
                midi_in: midi_io_input_index..(midi_io_input_index + midi_io_inputs),
                value_in: value_io_input_index..(value_io_input_index + value_io_inputs),
                stream_out: stream_io_output_index..(stream_io_output_index + stream_io_outputs),
                midi_out: midi_io_output_index..(midi_io_output_index + midi_io_outputs),
                value_out: value_io_output_index..(value_io_output_index + value_io_outputs),
                resources: io_setup.resources_index..(io_setup.resources_index + io_setup.resources_count),
            },
        );
    }

    Ok(Indexes {
        streams: stream_io,
        midis: midi_io,
        values: value_io,
        max_stream_channels,
        max_midi_channels,
        max_value_channels,
        stream_count,
        midi_count,
        value_count,
        node_io: node_mapped_io,
        resources_tracking: io_needed.resources_tracking.clone(),
    })
}
