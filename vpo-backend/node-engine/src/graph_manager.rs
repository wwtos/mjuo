use std::cell::RefCell;

use ddgg::{EdgeIndex, Graph, GraphDiff, GraphError, VertexIndex};
use serde::{Deserialize, Serialize};
use sound_engine::SoundConfig;

use crate::connection::{SocketDirection, SocketType};
use crate::errors::{NodeError, NodeOk, WarningBuilder};
use crate::node::{NodeInitState, NodeRow};
use crate::nodes::variants::{new_variant, NodeVariant};
use crate::state::Action;
use crate::traversal::traverser::Traverser;
use crate::{node::NodeIndex, node_graph::NodeGraph};

type GraphManagerDiff = GraphDiff<NodeGraphWrapper, ConnectedThrough>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GraphIndex(pub VertexIndex);
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GraphEdgeIndex(pub EdgeIndex);
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ConnectedThrough(pub NodeIndex);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GlobalNodeIndex {
    pub graph_index: GraphIndex,
    pub node_index: NodeIndex,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeGraphWrapper {
    pub graph: RefCell<NodeGraph>,
    #[serde(skip)]
    pub traverser: Traverser,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphManager {
    node_graphs: Graph<NodeGraphWrapper, ConnectedThrough>,
    root_index: GraphIndex,
}

impl GraphManager {
    pub fn new() -> Self {
        let graph = Graph::new();
        let (root_index, _) = graph
            .add_vertex(NodeGraphWrapper {
                graph: RefCell::new(NodeGraph::new()),
                traverser: Traverser::default(),
            })
            .unwrap();

        GraphManager {
            node_graphs: graph,
            root_index: GraphIndex(root_index),
        }
    }

    pub fn new_graph(&mut self) -> Result<(GraphIndex, GraphManagerDiff), NodeError> {
        let (graph_index, diff) = self.node_graphs.add_vertex(NodeGraphWrapper {
            graph: RefCell::new(NodeGraph::new()),
            traverser: Traverser::default(),
        })?;

        Ok((GraphIndex(graph_index), diff))
    }

    pub fn connect_graphs(
        &mut self,
        from: GraphIndex,
        through: ConnectedThrough,
        to: GraphIndex,
    ) -> Result<(GraphEdgeIndex, GraphManagerDiff), NodeError> {
        let (edge_index, diff) = self.node_graphs.add_edge(from.0, to.0, through)?;

        Ok((GraphEdgeIndex(edge_index), diff))
    }

    pub fn disconnect_graphs(
        &mut self,
        index: GraphEdgeIndex,
    ) -> Result<(ConnectedThrough, GraphManagerDiff), NodeError> {
        Ok(self.node_graphs.remove_edge(index.0)?)
    }

    pub fn get_connection_through(
        &mut self,
        from: GraphIndex,
        through: ConnectedThrough,
        to: GraphIndex,
    ) -> Result<GraphEdgeIndex, NodeError> {
        let shared_edges = self.node_graphs.shared_edges(from.0, to.0)?;

        for edge_index in shared_edges {
            let edge = self.node_graphs.get_edge(edge_index)?;

            if edge.data == through {
                return Ok(GraphEdgeIndex(edge_index));
            }
        }

        Err(NodeError::NotConnected)
    }

    pub fn disconnect_graphs_through(
        &mut self,
        from: GraphIndex,
        through: ConnectedThrough,
        to: GraphIndex,
    ) -> Result<(ConnectedThrough, GraphManagerDiff), NodeError> {
        let shared_edges = self.node_graphs.shared_edges(from.0, to.0)?;

        for edge_index in shared_edges {
            let edge = self.node_graphs.get_edge(edge_index)?;

            if edge.data == through {
                return Ok(self.node_graphs.remove_edge(shared_edges[0])?);
            }
        }

        Err(NodeError::NotConnected)
    }

    /// Will error out if there's more than one parent node connected
    pub fn remove_graph(
        &mut self,
        graph_index: &GraphIndex,
    ) -> Result<(NodeGraphWrapper, GraphManagerDiff), NodeError> {
        let parent_nodes = self.node_graphs.get_vertex(graph_index.0)?.get_connections_from();

        if !parent_nodes.is_empty() {
            Err(NodeError::GraphHasOtherParents)
        } else {
            Ok(self.node_graphs.remove_vertex(graph_index.0)?)
        }
    }

    pub fn get_graph(&self, index: GraphIndex) -> Result<&NodeGraphWrapper, NodeError> {
        Ok(self.node_graphs.get_vertex_data(index.0)?)
    }

    fn get_graph_mut(&self, index: GraphIndex) -> Result<&mut NodeGraphWrapper, NodeError> {
        Ok(self.node_graphs.get_vertex_data_mut(index.0)?)
    }

    pub fn get_graph_parents(&self, graph_index: GraphIndex) -> Result<Vec<GlobalNodeIndex>, NodeError> {
        let parents = self.node_graphs.get_vertex(graph_index.0)?.get_connections_from();

        parents
            .iter()
            .map(|(vertex_index, edge_index)| {
                self.node_graphs.get_edge_data(*edge_index).map(|edge| GlobalNodeIndex {
                    graph_index: GraphIndex(*vertex_index),
                    node_index: edge.0,
                })
            })
            .collect::<Result<Vec<GlobalNodeIndex>, GraphError>>()
            .map_err(|err| err.into())
    }

    pub fn recalculate_traversal_for_graph(&self, index: GraphIndex) -> Result<(), NodeError> {
        let mut graph_wrapper = self.get_graph(index)?;

        // set the new traverser
        graph_wrapper.traverser = Traverser::get_traverser(&graph_wrapper.graph.borrow());

        Ok(())
    }

    pub fn update_traversal_defaults(
        &self,
        index: GraphIndex,
        nodes_to_update: Vec<NodeIndex>,
    ) -> Result<(), NodeError> {
        let graph_wrapper = self.get_graph(index)?;

        let NodeGraphWrapper { traverser, graph, .. } = &mut *graph_wrapper;

        for node_index in nodes_to_update.iter() {
            traverser.update_node_defaults(&graph.borrow(), node_index);
        }

        Ok(())
    }
}

impl GraphManager {
    pub fn create_node_at_index(
        &mut self,
        node_type: &str,
        graph_index: GraphIndex,
        sound_config: &SoundConfig,
        state: NodeInitState,
    ) -> Result<NodeOk<Action>, NodeError> {
        let mut warnings = WarningBuilder::new();

        let NodeInitState {
            props,
            registry,
            script_engine,
            global_state,
        } = state;

        let new_node = new_variant(node_type, sound_config)?;

        let graph = &mut self.get_graph(graph_index)?.graph.borrow_mut();
        let creation_result = graph.add_node(
            new_node,
            NodeInitState {
                props,
                registry,
                script_engine,
                global_state,
            },
        )?;

        let new_node_index = creation_result.value.0;
        warnings.append_warnings(creation_result.warnings);

        let node_creation_diff = creation_result.value.1;
        let mut new_graph_diff = None;

        // now, for the issue of child graphs

        // does this node need a child graph?
        let new_node_wrapper = graph.get_node(new_node_index)?;
        let uses_child_graph = new_node_wrapper.uses_child_graph();

        let child_graph_index = if uses_child_graph {
            let new_graph_index = {
                // create a graph for it
                let (new_graph_index, creation_diff) = self.new_graph()?;
                new_graph_diff = Some(creation_diff);

                let new_node = graph.get_node_mut(new_node_index)?;

                // get a list of the input and output nodes in the child graph
                // (for creating the InputsNode and OutputsNode inside the child graph)
                let child_sockets = new_node.get_child_graph_socket_list(registry);

                let input_sockets = child_sockets
                    .iter()
                    .filter_map(|child_socket| {
                        if child_socket.1 == SocketDirection::Input {
                            Some(child_socket.0)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<SocketType>>();

                let output_sockets = child_sockets
                    .iter()
                    .filter_map(|child_socket| {
                        if child_socket.1 == SocketDirection::Output {
                            Some(child_socket.0)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<SocketType>>();

                // let the node's wrapper set up the graph
                new_node.init_child_graph(
                    graph_index,
                    self,
                    input_sockets,
                    output_sockets,
                    NodeInitState {
                        props,
                        registry,
                        script_engine,
                        global_state,
                    },
                );

                // run the node's graph init function
                let new_child_graph = &mut self.get_graph(new_graph_index)?.graph.borrow();
                new_node.node_init_graph(new_child_graph);

                new_graph_index
            };

            let (_, connect_diff) =
                self.connect_graphs(graph_index, ConnectedThrough(new_node_index), new_graph_index)?;

            Some(new_graph_index)
        } else {
            None
        };

        let graph = &mut self.get_graph(graph_index)?.graph.borrow_mut();
        let new_node = graph.get_node_mut(new_node_index)?;

        Ok(NodeOk::new(
            Action::CreateNode {
                node_type: node_type.to_string(),
                graph_index,
                node_index: Some(new_node_index),
                child_graph_index,
                child_graph_io_indexes: *new_node.get_child_graph_io_indexes(),
            },
            warnings.into_warnings(),
        ))
    }

    pub fn remove_node(&mut self, index: GlobalNodeIndex) -> Result<Action, NodeError> {
        let GlobalNodeIndex {
            graph_index,
            node_index,
        } = index;

        // first, see if the node has a child graph
        let children = self.node_graphs.get_vertex(graph_index.0)?.get_connections_to();

        if !children.is_empty() {
            let (_, disconnect_diff) =
                self.disconnect_graphs_through(graph_index, ConnectedThrough(node_index), GraphIndex(children[0].0))?;
        }

        // now that we've ensured that the graph is disconnected, we can safely delete the node
        // but first, we need to make a list of its current connections (for undo)
        let graph = self.get_graph(graph_index)?.graph.borrow_mut();
        let node = graph.get_node(node_index)?;

        // now, we can remove the node
        let (_, remove_diff) = graph.remove_node(node_index)?;

        Ok(Action::None)
    }
}

impl GraphManager {
    pub fn post_deserialization(&mut self, state: NodeInitState, sound_config: &SoundConfig) -> Result<(), NodeError> {
        let NodeInitState {
            props,
            registry,
            script_engine,
            global_state,
        } = state;

        for graph_wrapper_index in self.node_graphs.vertex_indexes() {
            let graph_wrapper = self.get_graph_mut(GraphIndex(graph_wrapper_index))?;

            graph_wrapper.graph.borrow_mut().post_deserialization(
                NodeInitState {
                    props,
                    registry,
                    script_engine,
                    global_state,
                },
                sound_config,
            )?;
            graph_wrapper.traverser = Traverser::get_traverser(&graph_wrapper.graph.borrow());
        }

        // next, init child graph inputs and outputs nodes
        for graph_wrapper_index in self.node_graphs.vertex_indexes() {
            let graph_wrapper = self.get_graph(GraphIndex(graph_wrapper_index))?;
            let graph = graph_wrapper.graph.borrow();

            for node_index in graph.node_indexes() {
                let node = graph.get_node_mut(node_index)?;
                let socket_list = node.get_child_graph_socket_list(registry);

                if let Some(index) = node.get_child_graph_index() {
                    let mut child_graph = self.get_graph(*index)?.graph.borrow_mut();

                    let input_sockets = socket_list
                        .iter()
                        .filter_map(|child_socket| {
                            if child_socket.1 == SocketDirection::Input {
                                Some(child_socket.0.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<SocketType>>();

                    let output_sockets = socket_list
                        .iter()
                        .filter_map(|child_socket| {
                            if child_socket.1 == SocketDirection::Output {
                                Some(child_socket.0.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<SocketType>>();

                    let (input_index, output_index) = node.get_child_graph_io_indexes().unwrap();

                    let input_node = child_graph.get_node_mut(input_index)?;
                    if let NodeVariant::InputsNode(inputs_node) = &mut input_node.node {
                        println!("\n\nsetting inputs: {:?}\n\n", input_sockets);
                        inputs_node.set_inputs(input_sockets.clone());
                    } else {
                        unreachable!("Child graph input index is not input node!");
                    }

                    input_node.set_node_rows(
                        input_sockets
                            .into_iter()
                            .map(|socket| NodeRow::from_type_and_direction(socket, SocketDirection::Output, false))
                            .collect(),
                    );

                    let output_node = child_graph.get_node_mut(output_index)?;
                    if let NodeVariant::OutputsNode(outputs_node) = &mut output_node.node {
                        println!("\n\nsetting outputs: {:?}\n\n", output_sockets);
                        outputs_node.set_outputs(output_sockets.clone());
                    }

                    output_node.set_node_rows(
                        output_sockets
                            .into_iter()
                            .map(|socket| NodeRow::from_type_and_direction(socket, SocketDirection::Input, false))
                            .collect(),
                    );
                }
            }
        }

        // finally go through and run init_child_graph for all the nodes in the root graph
        let root_graph = self.get_graph(self.root_index)?.graph.borrow_mut();

        for node_index in root_graph.node_indexes() {
            let node = root_graph.get_node(node_index)?;

            if let Some(child_graph_index) = node.get_child_graph_index() {
                let mut child_graph = self.get_graph(*child_graph_index)?;

                node.node_init_graph(&mut child_graph.graph.borrow_mut());
            }
        }

        Ok(())
    }
}
