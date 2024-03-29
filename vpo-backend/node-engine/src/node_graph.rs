use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use ddgg::{Edge, EdgeIndex, Graph, GraphDiff};
use serde::{Deserialize, Serialize};
use snafu::OptionExt;

use crate::{
    connection::{InputSideConnection, OutputSideConnection, Socket, SocketDirection},
    errors::{NodeDoesNotExistSnafu, NodeError, NodeOk, NodeResult, NodesNotConnectedSnafu},
    node::{NodeGetIoContext, NodeIndex},
    node_instance::NodeInstance,
    nodes::variant_io,
};

pub type NodeGraphDiff = GraphDiff<NodeInstance, NodeConnectionData>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NodeConnectionData {
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
    nodes: Graph<NodeInstance, NodeConnectionData>,
    #[serde(skip)]
    default_channel_count: usize,
}

pub(crate) fn create_new_node(node_type: &str, ctx: NodeGetIoContext) -> NodeResult<NodeInstance> {
    let node_rows = variant_io(node_type, ctx, HashMap::default())?.node_rows;

    let new_node = NodeInstance::new(node_type.into(), node_rows)?;

    Ok(NodeOk::new(new_node.value, new_node.warnings))
}

impl NodeGraph {
    pub fn new(default_channel_count: usize) -> NodeGraph {
        NodeGraph {
            nodes: Graph::new(),
            default_channel_count,
        }
    }

    pub fn add_node(&mut self, node_type: &str) -> NodeResult<(NodeIndex, NodeGraphDiff)> {
        let new_node = create_new_node(node_type, NodeGetIoContext::no_io_yet(self.default_channel_count))?;

        let (index, diff) = self.nodes.add_vertex(new_node.value);

        Ok(NodeOk::new((NodeIndex(index), diff), new_node.warnings))
    }

    pub(super) fn update_node_no_row_updates(
        &mut self,
        index: NodeIndex,
        node: NodeInstance,
    ) -> Result<NodeGraphDiff, NodeError> {
        Ok(self.nodes.update_vertex(index.0, node)?.1)
    }

    pub fn connect(
        &mut self,
        from_index: NodeIndex,
        from_socket: &Socket,
        to_index: NodeIndex,
        to_socket: &Socket,
    ) -> Result<(ConnectionIndex, NodeGraphDiff), NodeError> {
        // check that this connection doesn't already exist
        let existing_connection = self.get_input_connection_index(to_index, to_socket)?;

        if existing_connection.is_some() {
            return Err(NodeError::AlreadyConnected {
                from: from_socket.clone(),
                to: to_socket.clone(),
            });
        }

        let from = self
            .nodes
            .get_vertex(from_index.0)
            .with_context(|| NodeDoesNotExistSnafu { node_index: from_index })?
            .data();
        let to = self
            .nodes
            .get_vertex(to_index.0)
            .with_context(|| NodeDoesNotExistSnafu { node_index: to_index })?
            .data();

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
        if from_socket.socket_type() != to_socket.socket_type() || from_socket.channels() != to_socket.channels() {
            return Err(NodeError::IncompatibleSocketTypes {
                from: from_socket.socket_type(),
                to: to_socket.socket_type(),
            });
        }

        let (edge_index, graph_diff) = self.nodes.add_edge(
            from_index.0,
            to_index.0,
            NodeConnectionData {
                from_socket: from_socket.clone(),
                to_socket: to_socket.clone(),
            },
        )?;

        Ok((ConnectionIndex(edge_index), graph_diff))
    }

    pub fn disconnect(
        &mut self,
        from_index: NodeIndex,
        from_socket: &Socket,
        to_index: NodeIndex,
        to_socket: &Socket,
    ) -> Result<(NodeConnectionData, NodeGraphDiff), NodeError> {
        // check that the connection exists
        let edge_index = self.get_connection_index(from_index, from_socket, to_index, to_socket)?;

        Ok(self.nodes.remove_edge(edge_index.0)?)
    }

    pub fn disconnect_by_index(
        &mut self,
        edge_index: ConnectionIndex,
    ) -> Result<(NodeConnectionData, NodeGraphDiff), NodeError> {
        Ok(self.nodes.remove_edge(edge_index.0)?)
    }

    pub fn remove_node(&mut self, index: NodeIndex) -> Result<(NodeInstance, NodeGraphDiff), NodeError> {
        let (old_data, diff) = self.nodes.remove_vertex(index.0)?;

        Ok((old_data, diff))
    }

    pub fn get_connection_index(
        &self,
        from_index: NodeIndex,
        from_socket: &Socket,
        to_index: NodeIndex,
        to_socket: &Socket,
    ) -> Result<ConnectionIndex, NodeError> {
        let edges = self.nodes.shared_edges(from_index.0, to_index.0)?;

        for edge_index in edges {
            let edge = self.nodes[edge_index].data();

            if &edge.from_socket == from_socket && &edge.to_socket == to_socket {
                return Ok(ConnectionIndex(edge_index));
            }
        }

        Err(NodeError::NodesNotConnected {
            from_index,
            from_socket: from_socket.clone(),
            to_index,
            to_socket: to_socket.clone(),
        })
    }

    pub fn get_output_connection_indexes(
        &self,
        from_index: NodeIndex,
        from_socket: Socket,
    ) -> Result<Vec<ConnectionIndex>, NodeError> {
        let outgoing_edge_indexes = self
            .nodes
            .get_vertex(from_index.0)
            .with_context(|| NodeDoesNotExistSnafu { node_index: from_index })?
            .get_connections_to();

        let matching: Vec<ConnectionIndex> = outgoing_edge_indexes
            .iter()
            .map(|(_, edge_index)| {
                self.nodes
                    .get_edge(*edge_index)
                    .map(|edge| (edge.data(), edge_index))
                    .expect("edge to exist")
            })
            .filter(|(edge, _)| edge.from_socket == from_socket)
            .map(|(_, edge_index)| ConnectionIndex(*edge_index))
            .collect::<Vec<ConnectionIndex>>();

        Ok(matching)
    }

    pub fn get_input_connection_index(
        &self,
        to_index: NodeIndex,
        to_socket: &Socket,
    ) -> Result<Option<ConnectionIndex>, NodeError> {
        let edge_indexes = self
            .nodes
            .get_vertex(to_index.0)
            .with_context(|| NodeDoesNotExistSnafu { node_index: to_index })?
            .get_connections_from();

        let matching: Vec<ConnectionIndex> = edge_indexes
            .iter()
            .map(|(_, edge_index)| {
                self.nodes
                    .get_edge(*edge_index)
                    .map(|edge| (edge.data(), edge_index))
                    .expect("edge to exist")
            })
            .filter(|(edge, _)| &edge.to_socket == to_socket)
            .map(|(_, edge_index)| ConnectionIndex(*edge_index))
            .collect::<Vec<ConnectionIndex>>();

        Ok(matching.last().copied())
    }

    pub fn get_input_side_connections(&self, index: NodeIndex) -> Result<Vec<InputSideConnection>, NodeError> {
        let edge_indexes = self
            .nodes
            .get_vertex(index.0)
            .with_context(|| NodeDoesNotExistSnafu { node_index: index })?
            .get_connections_from();

        let matching: Vec<InputSideConnection> = edge_indexes
            .iter()
            .map(|(from_node, edge_index)| {
                let edge = self.nodes.get_edge(*edge_index).expect("edge to exist");
                InputSideConnection {
                    from_socket: edge.data().from_socket.clone(),
                    from_node: NodeIndex(*from_node),
                    to_socket: edge.data().to_socket.clone(),
                }
            })
            .collect::<Vec<InputSideConnection>>();

        Ok(matching)
    }

    pub fn get_output_side_connections(&self, from_index: NodeIndex) -> Result<Vec<OutputSideConnection>, NodeError> {
        let edge_indexes = self
            .nodes
            .get_vertex(from_index.0)
            .with_context(|| NodeDoesNotExistSnafu { node_index: from_index })?
            .get_connections_to();

        let matching: Vec<OutputSideConnection> = edge_indexes
            .iter()
            .map(|(to_node, edge_index)| {
                let edge = self.nodes.get_edge(*edge_index).expect("edge to exist");

                OutputSideConnection {
                    from_socket: edge.data().from_socket.clone(),
                    to_node: NodeIndex(*to_node),
                    to_socket: edge.data().to_socket.clone(),
                }
            })
            .collect::<Vec<OutputSideConnection>>();

        Ok(matching)
    }

    pub fn get_connection(
        &self,
        from_index: NodeIndex,
        from_socket: &Socket,
        to_index: NodeIndex,
        to_socket: &Socket,
    ) -> Result<&NodeConnectionData, NodeError> {
        let index = self.get_connection_index(from_index, from_socket, to_index, to_socket)?;

        Ok(self
            .nodes
            .get_edge(index.0)
            .with_context(|| NodesNotConnectedSnafu {
                from_index,
                from_socket: from_socket.clone(),
                to_index,
                to_socket: to_socket.clone(),
            })?
            .data())
    }

    pub fn get_node(&self, index: NodeIndex) -> Result<&NodeInstance, NodeError> {
        Ok(self
            .nodes
            .get_vertex_data(index.0)
            .with_context(|| NodeDoesNotExistSnafu { node_index: index })?)
    }

    pub fn get_node_mut(&mut self, index: NodeIndex) -> Result<&mut NodeInstance, NodeError> {
        Ok(self
            .nodes
            .get_vertex_data_mut(index.0)
            .with_context(|| NodeDoesNotExistSnafu { node_index: index })?)
    }

    pub fn node_indexes(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.nodes.vertex_indexes().map(NodeIndex)
    }

    pub fn edge_indexes(&self) -> impl Iterator<Item = ConnectionIndex> + '_ {
        self.nodes.edge_indexes().map(ConnectionIndex)
    }

    pub fn nodes_data_iter(&self) -> impl Iterator<Item = (NodeIndex, &NodeInstance)> + '_ {
        self.nodes
            .vertex_data_iter()
            .map(|(index, vertex)| (NodeIndex(index), vertex))
    }

    pub fn edges_iter(&self) -> impl Iterator<Item = (ConnectionIndex, &Edge<NodeConnectionData>)> + '_ {
        self.nodes
            .edge_iter()
            .map(|(index, edge)| (ConnectionIndex(index), edge))
    }

    pub fn get_graph(&self) -> &Graph<NodeInstance, NodeConnectionData> {
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

    pub fn set_default_channel_count(&mut self, default_channel_count: usize) {
        self.default_channel_count = default_channel_count;
    }
}

impl Index<NodeIndex> for NodeGraph {
    type Output = NodeInstance;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        self.get_node(index).unwrap()
    }
}

impl IndexMut<NodeIndex> for NodeGraph {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        self.get_node_mut(index).unwrap()
    }
}
