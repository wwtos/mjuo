use std::collections::{BTreeMap, HashMap};

use crate::connection::{Connection, Primitive, Socket, SocketType};
use crate::errors::{NodeError, NodeWarning};
use crate::global_state::{ResourceType, Resources};
use crate::graph_manager::GraphManager;
use crate::node::{NodeGetIoContext, NodeIndex, NodeInitParams, NodeRow, NodeRuntime};
use crate::node_graph::{NodeConnectionData, NodeGraph};
use crate::nodes::{new_variant, NodeVariant};

use ddgg::VertexIndex;
use petgraph::algo::{greedy_feedback_arc_set, toposort};
use petgraph::prelude::*;
use resource_manager::{ResourceId, ResourceIndex};
use rhai::Engine;
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
            edge.data.clone(),
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

pub(crate) struct NodeLayout {
    graph_index: NodeIndex,
    node: NodeVariant,
    to_input: Vec<(usize, Vec<Primitive>)>,
    midi_inputs: Vec<usize>,
    value_inputs: Vec<usize>,
    stream_inputs: Vec<usize>,
    midi_outputs: Vec<Socket>,
    value_outputs: Vec<Socket>,
    stream_outputs: Vec<Socket>,
    midi_index: usize,
    value_index: usize,
    stream_index: usize,
}

pub(crate) struct Layout {
    nodes: BTreeMap<NodeIndex, NodeLayout>,
    resources_tracking: Vec<(ResourceId, Option<(ResourceType, ResourceIndex)>)>,
    nodes_linked_to_ui: Vec<(usize, NodeIndex)>,
    traversal_order: Vec<NodeIndex>,
}

pub(crate) fn layout_and_init_nodes(
    graph: &NodeGraph,
    mut old_nodes: BTreeMap<NodeIndex, NodeVariant>,
    sound_config: &SoundConfig,
    script_engine: &Engine,
    resources: &Resources,
    current_time: i64,
    graph_manager: &GraphManager,
    default_channel_count: usize,
) -> Result<Layout, NodeError> {
    let traversal_order = calculate_graph_traverse_order(&graph);

    let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
    let mut warnings: Vec<(NodeIndex, NodeWarning)> = vec![];

    let mut resources_tracking: Vec<(ResourceId, Option<(ResourceType, ResourceIndex)>)> = vec![];
    let mut nodes_linked_to_ui: Vec<(usize, NodeIndex)> = vec![];

    let mut nodes: BTreeMap<NodeIndex, NodeLayout> = BTreeMap::new();

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

        for needed_resource in &needed_resources {
            let resource_index = resources.get_resource_index(needed_resource);

            resources_tracking.push((needed_resource.clone(), resource_index));
        }

        if variant.has_state() {
            nodes_linked_to_ui.push((vec_index, *node_index));
        }

        let mut stream_inputs: Vec<usize> = vec![];
        let mut midi_inputs: Vec<usize> = vec![];
        let mut value_inputs: Vec<usize> = vec![];

        let mut value_socket_to_index = vec![];

        let mut to_input: Vec<(usize, Vec<Primitive>)> = vec![];

        // go through the node by all its inputs
        for socket in node_instance.list_input_sockets() {
            let default_row = node_instance.get_default(socket).unwrap();

            if let NodeRow::Input(socket, default) = default_row {
                let is_connected = graph.get_input_connection_index(*node_index, &socket)?.is_some();

                match socket.socket_type() {
                    SocketType::Stream => {
                        stream_inputs.push(socket.channels());
                    }
                    SocketType::Midi => {
                        midi_inputs.push(socket.channels());
                    }
                    SocketType::Value => {
                        // if it's not connected to anything, be sure to input its default
                        if !is_connected {
                            to_input.push((value_inputs.len() - 1, vec![default.as_value().unwrap()]));
                        }

                        value_inputs.push(socket.channels());
                        value_socket_to_index.push((socket, value_inputs.clone()));
                    }
                    _ => {}
                }
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
            NodeLayout {
                graph_index: *node_index,
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
            },
        );

        midi_i += midi_output_sockets.iter().map(|x| x.channels()).sum::<usize>();
        value_i += value_output_sockets.iter().map(|x| x.channels()).sum::<usize>();
        stream_i += stream_output_sockets.iter().map(|x| x.channels()).sum::<usize>();
    }

    Ok(Layout {
        nodes,
        resources_tracking,
        nodes_linked_to_ui,
        traversal_order,
    })
}

#[test]
fn test_layout() {
    let mut graph = NodeGraph::new();
    let (gain, _) = graph.add_node("GainNode").unwrap().value;
    let (midi, _) = graph.add_node("MidiToValuesNode").unwrap().value;
    let (osc, _) = graph.add_node("OscillatorNode").unwrap().value;

    graph
        .connect(
            midi,
            &Socket::Simple("frequency".into(), SocketType::Value, 1),
            osc,
            &Socket::Simple("frequency".into(), SocketType::Value, 1),
        )
        .unwrap();
}
