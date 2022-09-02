use std::cell::{RefCell, RefMut};
use std::{cell::Ref, collections::HashMap};

use rhai::Engine;
use sound_engine::SoundConfig;

use crate::connection::{Connection, SocketDirection, SocketType};
use crate::errors::NodeError;
use crate::node_graph::create_new_node;
use crate::nodes::variants::new_variant;
use crate::socket_registry::SocketRegistry;
use crate::state::Action;
use crate::traversal::traverser::Traverser;
use crate::{node::NodeIndex, node_graph::NodeGraph};

pub type GraphIndex = u64;

#[derive(Debug, Clone)]
pub struct GlobalNodeIndex {
    pub graph_index: GraphIndex,
    pub node_index: NodeIndex,
}

#[derive(Debug)]
pub struct NodeGraphWrapper {
    pub graph: NodeGraph,
    pub traverser: Traverser,
    parent_nodes: Vec<GlobalNodeIndex>,
}

#[derive(Default, Debug)]
pub struct GraphManager {
    node_graphs: HashMap<u64, RefCell<NodeGraphWrapper>>,
    current_uid: u64,
}

impl GraphManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_graph(&mut self) -> GraphIndex {
        let graph_index = self.current_uid;
        self.current_uid += 1;

        self.node_graphs.insert(
            graph_index,
            RefCell::new(NodeGraphWrapper {
                graph: NodeGraph::new(),
                traverser: Traverser::default(),
                parent_nodes: vec![],
            }),
        );

        graph_index
    }

    pub(in crate) fn new_graph_unchecked(&mut self, graph_index: GraphIndex) -> GraphIndex {
        self.node_graphs.insert(
            graph_index,
            RefCell::new(NodeGraphWrapper {
                graph: NodeGraph::new(),
                traverser: Traverser::default(),
                parent_nodes: vec![],
            }),
        );

        graph_index
    }

    pub fn get_subgraph_parent_nodes(&self, subgraph_index: GraphIndex) -> Vec<GlobalNodeIndex> {
        self.get_graph_wrapper_ref(subgraph_index).unwrap().parent_nodes.clone()
    }

    pub fn add_parent_node(
        &mut self,
        child_graph: GraphIndex,
        graph_of_parent_node: GraphIndex,
        parent_node: NodeIndex,
    ) {
        let mut child_graph = self.get_graph_wrapper_mut(child_graph).unwrap();

        // TODO: verify `node_graph` is valid
        child_graph.parent_nodes.push(GlobalNodeIndex {
            graph_index: graph_of_parent_node,
            node_index: parent_node,
        });
    }

    pub fn remove_parent_node(
        &mut self,
        child_graph: GraphIndex,
        graph_of_parent_node: GraphIndex,
        parent_node: NodeIndex,
    ) -> Result<(), NodeError> {
        let mut child_graph = self
            .get_graph_wrapper_mut(child_graph)
            .ok_or(NodeError::GraphDoesNotExist(child_graph))?;

        let entry = child_graph
            .parent_nodes
            .iter()
            .position(|potential_parent_node| {
                potential_parent_node.node_index == parent_node
                    && potential_parent_node.graph_index == graph_of_parent_node
            })
            .ok_or(NodeError::GraphDoesNotExist(graph_of_parent_node))?;

        child_graph.parent_nodes.remove(entry);

        Ok(())
    }

    pub fn get_graph_wrapper_ref(&self, index: GraphIndex) -> Option<Ref<NodeGraphWrapper>> {
        self.node_graphs.get(&index).map(|x| (*x).borrow())
    }

    pub fn get_graph_wrapper_mut(&self, index: GraphIndex) -> Option<RefMut<NodeGraphWrapper>> {
        self.node_graphs.get(&index).map(|x| (*x).borrow_mut())
    }

    pub fn recalculate_traversal_for_graph(&self, index: &GraphIndex) -> Result<(), NodeError> {
        let mut graph_wrapper = self
            .get_graph_wrapper_mut(*index)
            .ok_or(NodeError::GraphDoesNotExist(*index))?;

        // set the new traverser
        graph_wrapper.traverser = Traverser::get_traverser(&graph_wrapper.graph);

        Ok(())
    }

    pub fn update_traversal_defaults(&self, index: GraphIndex, nodes_to_update: Vec<NodeIndex>) {
        let graph_wrapper = self.get_graph_wrapper_mut(index);

        if let Some(mut graph_wrapper) = graph_wrapper {
            let NodeGraphWrapper { traverser, graph, .. } = &mut *graph_wrapper;

            for node_index in nodes_to_update.iter() {
                traverser.update_node_defaults(graph, node_index);
            }
        }
    }

    /// Will error out if there's more than one node connected
    pub fn remove_graph(&mut self, graph_index: &GraphIndex) -> Result<(), NodeError> {
        let number_of_parent_nodes = {
            let graph = self
                .get_graph_wrapper_mut(*graph_index)
                .ok_or(NodeError::GraphDoesNotExist(*graph_index))?;
            graph.parent_nodes.len()
        };

        if number_of_parent_nodes > 1 {
            Err(NodeError::GraphHasOtherParents)
        } else {
            self.node_graphs.remove(graph_index);

            Ok(())
        }
    }
}

impl GraphManager {
    pub fn create_node_at_index(
        &mut self,
        node_type: &str,
        graph_index: GraphIndex,
        node_index: Option<NodeIndex>,
        child_graph_index: Option<GraphIndex>,
        sound_config: &SoundConfig,
        registry: &mut SocketRegistry,
        engine: &Engine,
    ) -> Result<Action, NodeError> {
        let new_node_index = {
            let graph = &mut self.get_graph_wrapper_mut(graph_index).unwrap().graph;

            // if it's a redo, it has a specific index it needs to be at
            if let Some(node_index) = node_index {
                if let Some(_) = graph.get_node(&node_index) {
                    return Err(NodeError::NodeAlreadyExists(node_index));
                }

                let new_node = new_variant(node_type, sound_config).unwrap();

                let new_node_wrapper = create_new_node(new_node, node_index.generation, registry, engine);

                graph.set_node_unchecked(node_index, new_node_wrapper);

                node_index
            } else {
                // else, it's happening for the first time
                let new_node = new_variant(node_type, sound_config).unwrap();

                let new_node_index = graph.add_node(new_node, registry, engine);

                new_node_index
            }
        };

        // now, for the issue of child graphs

        // if this is a redo
        let child_graph_index = if let Some(node_index) = node_index {
            // did it previously have a child graph?
            if let Some(child_graph_index) = child_graph_index {
                // if so, create it at the previous index (if it doesn't already exist)
                if self.get_graph_wrapper_ref(child_graph_index).is_none() {
                    self.new_graph_unchecked(child_graph_index);
                }

                // link them to each other
                self.add_parent_node(child_graph_index, graph_index, node_index);

                let graph = &mut self.get_graph_wrapper_mut(graph_index).unwrap().graph;
                graph
                    .get_node_mut(&node_index)
                    .unwrap()
                    .set_inner_graph_index(child_graph_index);

                Some(child_graph_index)
            } else {
                None
            }
        } else {
            // does this node need a child graph?
            let does_need_inner_graph_created = {
                let graph = &mut self.get_graph_wrapper_mut(graph_index).unwrap().graph;
                let new_node_wrapper = graph.get_node(&new_node_index).unwrap();

                new_node_wrapper.does_need_inner_graph_created()
            };

            if does_need_inner_graph_created {
                let new_graph_index = {
                    // create a graph for it
                    let new_graph_index = self.new_graph();

                    let graph = &mut self.get_graph_wrapper_mut(graph_index).unwrap().graph;
                    let new_node = graph.get_node_mut(&new_node_index).unwrap();

                    // get a list of the input and output nodes in the child graph
                    // (for creating the InputsNode and OutputsNode inside the child graph)
                    let (input_sockets, output_sockets) = {
                        let inner_sockets = new_node.get_inner_graph_socket_list(registry);

                        (
                            inner_sockets
                                .iter()
                                .filter_map(|inner_socket| {
                                    if inner_socket.1 == SocketDirection::Input {
                                        Some(inner_socket.0.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<SocketType>>(),
                            inner_sockets
                                .iter()
                                .filter_map(|inner_socket| {
                                    if inner_socket.1 == SocketDirection::Output {
                                        Some(inner_socket.0.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<SocketType>>(),
                        )
                    };

                    // let the node's wrapper set up the graph
                    new_node.init_inner_graph(&new_graph_index, self, input_sockets, output_sockets, registry, engine);

                    // run the node's graph init function
                    let new_inner_graph = &mut self.get_graph_wrapper_mut(new_graph_index).unwrap().graph;
                    new_node.node_init_graph(new_inner_graph);

                    new_graph_index
                };

                self.add_parent_node(new_graph_index, graph_index, new_node_index);

                Some(new_graph_index)
            } else {
                None
            }
        };

        Ok(Action::CreateNode {
            node_type: node_type.to_string(),
            graph_index: graph_index,
            node_index: Some(new_node_index),
            child_graph_index,
        })
    }

    pub fn remove_node(&mut self, index: &GlobalNodeIndex) -> Result<Action, NodeError> {
        let GlobalNodeIndex {
            graph_index,
            node_index,
        } = index;

        // first, see if the node is linked to a child graph
        let possible_child_graph_index = {
            let graph = self
                .get_graph_wrapper_mut(*graph_index)
                .ok_or(NodeError::GraphDoesNotExist(*graph_index))?;
            let node = graph
                .graph
                .get_node(node_index)
                .ok_or(NodeError::NodeDoesNotExist(*node_index))?;

            node.get_child_graph_index().clone()
        };

        if let Some(child_graph_index) = possible_child_graph_index {
            // we need to unlink that graph
            self.remove_parent_node(child_graph_index, *graph_index, *node_index)?;
        }

        // now that we've ensured that the graph is disconnected, we can safely delete the node
        // but first, we need to make a list of its current connections (for undo)
        let mut graph = self.get_graph_wrapper_mut(*graph_index).unwrap();
        let node = graph.graph.get_node(node_index).expect("Node already checked");

        // get everything useful from the node before deleting it
        let node_type = node.get_node_type();

        // make a list of current connections
        let node_connections_input = node.list_connected_input_sockets();
        let node_connections_output = node.list_connected_output_sockets();

        let node_connections = [
            node_connections_input
                .into_iter()
                .map(|input| Connection {
                    from_socket_type: input.from_socket_type,
                    from_node: input.from_node,
                    to_socket_type: input.to_socket_type,
                    to_node: *node_index,
                })
                .collect::<Vec<Connection>>(),
            node_connections_output
                .into_iter()
                .map(|output| Connection {
                    from_socket_type: output.from_socket_type,
                    from_node: *node_index,
                    to_socket_type: output.to_socket_type,
                    to_node: output.to_node,
                })
                .collect::<Vec<Connection>>(),
        ]
        .concat();

        // also, save the properties, value overrides, etc
        let node_state = node.serialize_to_json()?;

        // now, we can remove the node
        graph.graph.remove_node(node_index)?;

        Ok(Action::RemoveNode {
            node_type: Some(node_type),
            index: GlobalNodeIndex {
                graph_index: *graph_index,
                node_index: *node_index,
            },
            child_graph_index: possible_child_graph_index,
            connections: Some(node_connections),
            serialized: Some(node_state),
        })
    }
}
