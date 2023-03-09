use std::mem;

use ddgg::{EdgeIndex, Graph, GraphDiff, GraphError, VertexIndex};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sound_engine::SoundConfig;

use crate::{
    connection::{InputSideConnection, OutputSideConnection, SocketDirection, SocketType},
    errors::{NodeError, NodeOk, NodeResult, WarningBuilder},
    node::{Node, NodeIndex, NodeInitState, NodeRow, NodeWrapper},
    nodes::variants::NodeVariant,
};

pub type NodeGraphDiff = GraphDiff<NodeWrapper, NodeConnection>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NodeConnection {
    pub from_socket_type: SocketType,
    pub to_socket_type: SocketType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionIndex(pub EdgeIndex);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
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
        let existing_connection = self.get_input_connection_index(to_index, to_socket_type)?;

        if let Some(_) = existing_connection {
            return Err(NodeError::AlreadyConnected {
                from: from_socket_type,
                to: to_socket_type,
            });
        }

        let from = &self.nodes.get_vertex(from_index.0)?.data;
        let to = &self.nodes.get_vertex(to_index.0)?.data;

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

    pub fn disconnect_by_index(
        &mut self,
        edge_index: ConnectionIndex,
    ) -> Result<(NodeConnection, NodeGraphDiff), NodeError> {
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
            let edge = &self.nodes.get_edge(edge_index)?.data;

            if edge.from_socket_type == from_socket_type && edge.to_socket_type == to_socket_type {
                return Ok(ConnectionIndex(edge_index));
            }
        }

        Err(NodeError::NotConnected)
    }

    pub fn get_output_connection_indexes(
        &self,
        from_index: NodeIndex,
        to_socket_type: SocketType,
    ) -> Result<Vec<ConnectionIndex>, NodeError> {
        let edge_indexes = self.nodes.get_vertex(from_index.0)?.get_connections_to();

        let matching: Vec<ConnectionIndex> = edge_indexes
            .iter()
            .map(|(_, edge_index)| self.nodes.get_edge(*edge_index).map(|edge| (&edge.data, edge_index)))
            .filter_ok(|(edge, _)| edge.to_socket_type == to_socket_type)
            .map(|result| result.map(|(_, edge_index)| ConnectionIndex(*edge_index)))
            .collect::<Result<Vec<ConnectionIndex>, GraphError>>()?;

        Ok(matching)
    }

    pub fn get_input_connection_index(
        &self,
        to_index: NodeIndex,
        to_socket_type: SocketType,
    ) -> Result<Option<ConnectionIndex>, NodeError> {
        let edge_indexes = self.nodes.get_vertex(to_index.0)?.get_connections_from();

        let matching: Vec<ConnectionIndex> = edge_indexes
            .iter()
            .map(|(_, edge_index)| self.nodes.get_edge(*edge_index).map(|edge| (&edge.data, edge_index)))
            .filter_ok(|(edge, _)| edge.to_socket_type == to_socket_type)
            .map(|result| result.map(|(_, edge_index)| ConnectionIndex(*edge_index)))
            .collect::<Result<Vec<ConnectionIndex>, GraphError>>()?;

        Ok(matching.last().map(|index| *index))
    }

    pub fn get_input_side_connections(&self, from_index: NodeIndex) -> Result<Vec<InputSideConnection>, NodeError> {
        let edge_indexes = self.nodes.get_vertex(from_index.0)?.get_connections_from();

        let matching: Vec<InputSideConnection> = edge_indexes
            .iter()
            .map(|(from_node, edge_index)| {
                self.nodes.get_edge(*edge_index).map(|edge| InputSideConnection {
                    from_socket_type: edge.data.from_socket_type,
                    from_node: NodeIndex(*from_node),
                    to_socket_type: edge.data.to_socket_type,
                })
            })
            .collect::<Result<Vec<InputSideConnection>, GraphError>>()?;

        Ok(matching)
    }

    pub fn get_output_side_connections(&self, from_index: NodeIndex) -> Result<Vec<OutputSideConnection>, NodeError> {
        let edge_indexes = self.nodes.get_vertex(from_index.0)?.get_connections_to();

        let matching: Vec<OutputSideConnection> = edge_indexes
            .iter()
            .map(|(to_node, edge_index)| {
                self.nodes.get_edge(*edge_index).map(|edge| OutputSideConnection {
                    from_socket_type: edge.data.from_socket_type,
                    to_node: NodeIndex(*to_node),
                    to_socket_type: edge.data.to_socket_type,
                })
            })
            .collect::<Result<Vec<OutputSideConnection>, GraphError>>()?;

        Ok(matching)
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
    ) -> Result<NodeOk<(bool, Vec<GraphDiff<NodeWrapper, NodeConnection>>)>, NodeError> {
        let mut has_changed_self = false;
        let mut warnings = WarningBuilder::new();
        let mut diffs: Vec<GraphDiff<NodeWrapper, NodeConnection>> = Vec::new();

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

                // go through all the connections that were connected to this row and disconnect them
                if let Some((socket_type, direction)) = type_and_direction {
                    match direction {
                        SocketDirection::Input => {
                            let input_connection = self.get_input_connection_index(index, socket_type)?;

                            if let Some(input_connection) = input_connection {
                                let (_, diff) = self.disconnect_by_index(input_connection)?;
                                diffs.push(diff);
                            }
                        }
                        SocketDirection::Output => {
                            let output_connections = self.get_output_connection_indexes(index, socket_type)?;

                            for output_connection in output_connections {
                                let (_, diff) = self.disconnect_by_index(output_connection)?;
                                diffs.push(diff);
                            }
                        }
                    }
                }
            }

            // at which point we can _finally_ update the node's row list
            self.get_node_mut(index).unwrap().set_node_rows(new_node_rows);
            has_changed_self = true;
        }

        Ok(NodeOk::new((has_changed_self, diffs), warnings.into_warnings()))
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

    pub fn edge_indexes(&self) -> impl Iterator<Item = ConnectionIndex> + '_ {
        self.nodes.edge_indexes().map(|index| ConnectionIndex(index))
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
    pub fn post_deserialization(&mut self, state: NodeInitState, _sound_config: &SoundConfig) -> Result<(), NodeError> {
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
            .collect::<Vec<VertexIndex>>()
            .into_iter()
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
                )?;

                Ok(())
            })
            .collect::<Result<Vec<()>, NodeError>>()?;

        Ok(())
    }
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}
