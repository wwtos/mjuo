use std::mem;

use ddgg::{EdgeIndex, Graph, GraphDiff};
use serde::{Deserialize, Serialize};
use sound_engine::SoundConfig;

use crate::{
    connection::{OutputSideConnection, SocketDirection, SocketType},
    errors::{NodeError, NodeOk, NodeResult, WarningBuilder},
    node::{Node, NodeIndex, NodeInitState, NodeRow, NodeWrapper},
    nodes::variants::NodeVariant,
};

pub type NodeGraphDiff = GraphDiff<NodeWrapper, NodeConnection>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct NodeConnection {
    from_socket_type: SocketType,
    to_socket_type: SocketType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct ConnectionIndex(pub EdgeIndex);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeGraph {
    nodes: Graph<NodeWrapper, NodeConnection>,
}

pub(crate) fn create_new_node(node: NodeVariant, state: NodeInitState) -> NodeResult<NodeWrapper> {
    let new_node = NodeWrapper::new(node, state)?;

    Ok(NodeOk::new(new_node.value, new_node.warnings))
}

impl NodeGraph {
    pub fn new() -> NodeGraph {
        NodeGraph { nodes: Graph::new() }
    }

    pub fn add_node(&mut self, node: NodeVariant, state: NodeInitState) -> NodeResult<(NodeIndex, NodeGraphDiff)> {
        let new_node = create_new_node(node, state)?;

        let (index, diff) = self.nodes.add_vertex(new_node.value)?;

        Ok(NodeOk::new((NodeIndex(index), diff), new_node.warnings))
    }

    pub fn connect(
        &mut self,
        from_index: NodeIndex,
        from_socket_type: SocketType,
        to_index: NodeIndex,
        to_socket_type: SocketType,
    ) -> Result<(ConnectionIndex, NodeGraphDiff), NodeError> {
        // check that this connection doesn't already exist
        let existing_connection = self.get_connection(from_index, from_socket_type, to_index, to_socket_type);

        if existing_connection.is_ok() {
            return Err(NodeError::AlreadyConnected {
                from: from_socket_type,
                to: to_socket_type,
            });
        }

        let from = self.nodes.get_vertex(from_index.0)?.data;
        let to = self.nodes.get_vertex(to_index.0)?.data;

        if !from.has_output_socket(from_socket_type) {
            return Err(NodeError::SocketDoesNotExist {
                socket_type: from_socket_type.clone(),
            });
        }

        if !to.has_input_socket(to_socket_type) {
            return Err(NodeError::SocketDoesNotExist {
                socket_type: to_socket_type.clone(),
            });
        }

        // make sure the types are of the same family (midi can't connect to stream, etc)
        if mem::discriminant(&from_socket_type) != mem::discriminant(&to_socket_type) {
            return Err(NodeError::IncompatibleSocketTypes {
                from: from_socket_type.clone(),
                to: to_socket_type.clone(),
            });
        }

        let (edge_index, graph_diff) = self.nodes.add_edge(
            from_index.0,
            to_index.0,
            NodeConnection {
                from_socket_type,
                to_socket_type,
            },
        )?;

        Ok((ConnectionIndex(edge_index), graph_diff))
    }

    pub fn disconnect(
        &mut self,
        from_index: NodeIndex,
        from_socket_type: SocketType,
        to_index: NodeIndex,
        to_socket_type: SocketType,
    ) -> Result<(NodeConnection, NodeGraphDiff), NodeError> {
        // check that the connection exists
        let edge_index = self.get_connection_index(from_index, from_socket_type, to_index, to_socket_type)?;

        Ok(self.nodes.remove_edge(edge_index.0)?)
    }

    pub fn remove_node(&mut self, index: NodeIndex) -> Result<(NodeWrapper, NodeGraphDiff), NodeError> {
        let (old_data, diff) = self.nodes.remove_vertex(index.0)?;

        Ok((old_data, diff))
    }

    pub fn get_connection_index(
        &self,
        from_index: NodeIndex,
        from_socket_type: SocketType,
        to_index: NodeIndex,
        to_socket_type: SocketType,
    ) -> Result<ConnectionIndex, NodeError> {
        let edges = self.nodes.shared_edges(from_index.0, to_index.0)?;

        for edge_index in edges {
            let edge = self.nodes.get_edge(edge_index)?.data;

            if edge.from_socket_type == from_socket_type && edge.to_socket_type == to_socket_type {
                return Ok(ConnectionIndex(edge_index));
            }
        }

        Err(NodeError::NotConnected)
    }

    pub fn get_connection(
        &self,
        from_index: NodeIndex,
        from_socket_type: SocketType,
        to_index: NodeIndex,
        to_socket_type: SocketType,
    ) -> Result<&NodeConnection, NodeError> {
        let index = self.get_connection_index(from_index, from_socket_type, to_index, to_socket_type)?;

        Ok(&self.nodes.get_edge(index.0)?.data)
    }

    /// Initializes a node
    ///
    /// Returns whether the node has changed itself
    pub fn init_node(
        &mut self,
        index: NodeIndex,
        state: NodeInitState,
        force_update: bool,
    ) -> Result<NodeOk<bool>, NodeError> {
        let mut has_changed_self = false;
        let mut warnings = WarningBuilder::new();

        // will return the new node rows, if they changed
        let node_wrapper = self.get_node_mut(index)?;
        let possible_rows = {
            let NodeInitState {
                props: _,
                registry,
                script_engine,
                global_state,
            } = state;

            let props = node_wrapper.get_properties().clone();

            let node = &mut node_wrapper.node;
            let init_result = node.init(NodeInitState {
                props: &props,
                registry,
                script_engine,
                global_state,
            })?;

            warnings.append_warnings(init_result.warnings);

            // if the node returned any properties it wanted to change, apply them here
            if let Some(new_props) = init_result.value.changed_properties {
                for (key, prop) in new_props.into_iter() {
                    node_wrapper.set_property(key, prop);
                }
            }

            // return a list of all the rows that changed to the outer scope
            if init_result.value.did_rows_change || force_update {
                let old_rows = node_wrapper.get_node_rows().clone();
                let new_rows = &init_result.value.node_rows;

                // TODO: implement sockets changing properly
                // aka, if a socket is removed, safely disconnect it from the
                // other node

                // The main thing here is to see what properties were removed -- if it was removed
                // it needs to be disconnected safely
                let removed_rows: Vec<NodeRow> = old_rows
                    .iter()
                    .filter(|old_row| {
                        // if it's not in the new row, but it was in the old row,
                        // it's been removed
                        !new_rows.iter().any(|new_row| new_row == *old_row)
                    })
                    .cloned()
                    .collect();

                Some((removed_rows, init_result.value.node_rows))
            } else {
                None
            }
        };

        if let Some((removed_rows, new_node_rows)) = possible_rows {
            for removed_row in removed_rows {
                let type_and_direction = removed_row.to_type_and_direction();

                if let Some(type_and_direction) = type_and_direction {
                    let (socket_type, direction) = type_and_direction;

                    match direction {
                        SocketDirection::Input => {
                            let node_wrapper = self.get_node(index).unwrap();
                            let input_connection = node_wrapper.get_input_connection_by_type(&socket_type);

                            if let Some(input_connection) = input_connection {
                                let from_wrapper = self.get_node_mut(input_connection.from_node)?;

                                from_wrapper.remove_output_socket_connection_unchecked(&OutputSideConnection {
                                    from_socket_type: input_connection.from_socket_type,
                                    to_node: index,
                                    to_socket_type: input_connection.to_socket_type.clone(),
                                })?;

                                self.get_node_mut(index)
                                    .unwrap()
                                    .remove_input_socket_connection_unchecked(&input_connection.to_socket_type)?;
                            }
                        }
                        SocketDirection::Output => {
                            let node_wrapper = self.get_node(index).unwrap();
                            let output_connections = node_wrapper.get_output_connections_by_type(&socket_type);

                            for output_connection in output_connections {
                                let to_wrapper = self.get_node_mut(output_connection.to_node)?;
                                // remove the other connection to this one
                                to_wrapper
                                    .remove_input_socket_connection_unchecked(&output_connection.to_socket_type)?;

                                // remove this connection to the other one
                                self.get_node_mut(index)
                                    .unwrap()
                                    .remove_output_socket_connection_unchecked(&OutputSideConnection {
                                        from_socket_type: output_connection.from_socket_type,
                                        to_node: output_connection.to_node,
                                        to_socket_type: output_connection.to_socket_type,
                                    })?;
                            }
                        }
                    }
                }
            }

            // at which point we can _finally_ update the node's row list
            self.get_node_mut(index).unwrap().set_node_rows(new_node_rows);
            has_changed_self = true;
        }

        Ok(NodeOk::new(has_changed_self, warnings.into_warnings()))
    }

    pub fn get_node(&self, index: NodeIndex) -> Result<&NodeWrapper, NodeError> {
        Ok(self.nodes.get_vertex_data(index.0)?)
    }

    pub fn get_node_mut(&mut self, index: NodeIndex) -> Result<&mut NodeWrapper, NodeError> {
        Ok(self.nodes.get_vertex_data_mut(index.0)?)
    }

    pub fn node_indexes(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.nodes.vertex_indexes().map(|index| NodeIndex(index))
    }

    pub fn get_graph(&self) -> &Graph<NodeWrapper, NodeConnection> {
        &self.nodes
    }

    pub fn len(&self) -> usize {
        self.nodes.get_verticies().len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.get_verticies().is_empty()
    }

    pub fn apply_diff(&mut self, diff: NodeGraphDiff) -> Result<(), NodeError> {
        self.nodes.apply_diff(diff)?;

        Ok(())
    }

    pub fn rollback_diff(&mut self, diff: NodeGraphDiff) -> Result<(), NodeError> {
        self.nodes.rollback_diff(diff)?;

        Ok(())
    }
}

impl NodeGraph {
    pub fn post_deserialization(&mut self, state: NodeInitState, sound_config: &SoundConfig) -> Result<(), NodeError> {
        let NodeInitState {
            props,
            registry,
            script_engine,
            global_state,
        } = state;

        // go through and run post_deserialization on each node
        // then, initialize all those nodes in the graph here
        self.nodes
            .vertex_indexes()
            .map(|node_index| {
                self.init_node(
                    NodeIndex(node_index),
                    NodeInitState {
                        props,
                        registry,
                        script_engine,
                        global_state,
                    },
                    false,
                )
            })
            .collect::<Result<Vec<_>, NodeError>>()?;

        Ok(())
    }
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}
