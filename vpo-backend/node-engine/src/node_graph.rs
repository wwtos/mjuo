use std::collections::HashMap;

use ddgg::{EdgeIndex, Graph, GraphDiff, GraphError};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    connection::{InputSideConnection, OutputSideConnection, Socket},
    errors::{NodeError, NodeOk, NodeResult},
    node::NodeIndex,
    node_wrapper::NodeWrapper,
    nodes::variants::variant_io,
    socket_registry::SocketRegistry,
};

pub type NodeGraphDiff = GraphDiff<NodeWrapper, NodeConnection>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NodeConnection {
    pub from_socket: Socket,
    pub to_socket: Socket,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionIndex(pub EdgeIndex);

/// NodeGraph
///
/// This is the main structure for describing the graph (see traverser for actually processing the graph)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeGraph {
    nodes: Graph<NodeWrapper, NodeConnection>,
}

pub(crate) fn create_new_node(node_type: String, registry: &mut SocketRegistry) -> NodeResult<NodeWrapper> {
    let node_rows = variant_io(&node_type, HashMap::new(), &mut |name: &str| {
        registry.register_socket(name)
    })?
    .node_rows;
    let new_node = NodeWrapper::new(node_type, node_rows)?;

    Ok(NodeOk::new(new_node.value, new_node.warnings))
}

impl NodeGraph {
    pub fn new() -> NodeGraph {
        NodeGraph { nodes: Graph::new() }
    }

    pub fn add_node(
        &mut self,
        node_type: String,
        registry: &mut SocketRegistry,
    ) -> NodeResult<(NodeIndex, NodeGraphDiff)> {
        let new_node = create_new_node(node_type, registry)?;

        let (index, diff) = self.nodes.add_vertex(new_node.value)?;

        Ok(NodeOk::new((NodeIndex(index), diff), new_node.warnings))
    }

    pub fn connect(
        &mut self,
        from_index: NodeIndex,
        from_socket: Socket,
        to_index: NodeIndex,
        to_socket: Socket,
    ) -> Result<(ConnectionIndex, NodeGraphDiff), NodeError> {
        // check that this connection doesn't already exist
        let existing_connection = self.get_input_connection_index(to_index, to_socket)?;

        if let Some(_) = existing_connection {
            return Err(NodeError::AlreadyConnected {
                from: from_socket.to_owned(),
                to: to_socket.to_owned(),
            });
        }

        let from = &self.nodes.get_vertex(from_index.0)?.data;
        let to = &self.nodes.get_vertex(to_index.0)?.data;

        if !from.has_output_socket(from_socket) {
            return Err(NodeError::SocketDoesNotExist {
                socket: from_socket.to_owned(),
            });
        }

        if !to.has_input_socket(to_socket) {
            return Err(NodeError::SocketDoesNotExist {
                socket: to_socket.to_owned(),
            });
        }

        // make sure the types are of the same family (midi can't connect to stream, etc)
        if from_socket.socket_type() != to_socket.socket_type() {
            return Err(NodeError::IncompatibleSocketTypes {
                from: from_socket.socket_type(),
                to: to_socket.socket_type(),
            });
        }

        let (edge_index, graph_diff) =
            self.nodes
                .add_edge(from_index.0, to_index.0, NodeConnection { from_socket, to_socket })?;

        Ok((ConnectionIndex(edge_index), graph_diff))
    }

    pub fn disconnect(
        &mut self,
        from_index: NodeIndex,
        from_socket: Socket,
        to_index: NodeIndex,
        to_socket: Socket,
    ) -> Result<(NodeConnection, NodeGraphDiff), NodeError> {
        // check that the connection exists
        let edge_index = self.get_connection_index(from_index, from_socket, to_index, to_socket)?;

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
        from_socket: Socket,
        to_index: NodeIndex,
        to_socket: Socket,
    ) -> Result<ConnectionIndex, NodeError> {
        let edges = self.nodes.shared_edges(from_index.0, to_index.0)?;

        for edge_index in edges {
            let edge = &self.nodes.get_edge(edge_index)?.data;

            if edge.from_socket == from_socket && edge.to_socket == to_socket {
                return Ok(ConnectionIndex(edge_index));
            }
        }

        Err(NodeError::NotConnected)
    }

    pub fn get_output_connection_indexes(
        &self,
        from_index: NodeIndex,
        from_socket: Socket,
    ) -> Result<Vec<ConnectionIndex>, NodeError> {
        let edge_indexes = self.nodes.get_vertex(from_index.0)?.get_connections_to();

        let matching: Vec<ConnectionIndex> = edge_indexes
            .iter()
            .map(|(_, edge_index)| self.nodes.get_edge(*edge_index).map(|edge| (&edge.data, edge_index)))
            .filter_ok(|(edge, _)| edge.from_socket == from_socket)
            .map(|result| result.map(|(_, edge_index)| ConnectionIndex(*edge_index)))
            .collect::<Result<Vec<ConnectionIndex>, GraphError>>()?;

        Ok(matching)
    }

    pub fn get_input_connection_index(
        &self,
        to_index: NodeIndex,
        to_socket: Socket,
    ) -> Result<Option<ConnectionIndex>, NodeError> {
        let edge_indexes = self.nodes.get_vertex(to_index.0)?.get_connections_from();

        let matching: Vec<ConnectionIndex> = edge_indexes
            .iter()
            .map(|(_, edge_index)| self.nodes.get_edge(*edge_index).map(|edge| (&edge.data, edge_index)))
            .filter_ok(|(edge, _)| edge.to_socket == to_socket)
            .map(|result| result.map(|(_, edge_index)| ConnectionIndex(*edge_index)))
            .collect::<Result<Vec<ConnectionIndex>, GraphError>>()?;

        Ok(matching.last().map(|index| *index))
    }

    pub fn get_input_side_connections(&self, index: NodeIndex) -> Result<Vec<InputSideConnection>, NodeError> {
        let edge_indexes = self.nodes.get_vertex(index.0)?.get_connections_from();

        let matching: Vec<InputSideConnection> = edge_indexes
            .iter()
            .map(|(from_node, edge_index)| {
                self.nodes.get_edge(*edge_index).map(|edge| InputSideConnection {
                    from_socket: edge.data.from_socket,
                    from_node: NodeIndex(*from_node),
                    to_socket: edge.data.to_socket,
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
                    from_socket: edge.data.from_socket,
                    to_node: NodeIndex(*to_node),
                    to_socket: edge.data.to_socket,
                })
            })
            .collect::<Result<Vec<OutputSideConnection>, GraphError>>()?;

        Ok(matching)
    }

    pub fn get_connection(
        &self,
        from_index: NodeIndex,
        from_socket_type: Socket,
        to_index: NodeIndex,
        to_socket_type: Socket,
    ) -> Result<&NodeConnection, NodeError> {
        let index = self.get_connection_index(from_index, from_socket_type, to_index, to_socket_type)?;

        Ok(&self.nodes.get_edge(index.0)?.data)
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

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}
