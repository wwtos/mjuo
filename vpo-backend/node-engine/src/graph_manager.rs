use common::SeaHashMap;
use ddgg::{EdgeIndex, Graph, GraphDiff, VertexIndex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::{OptionExt, ResultExt};

use crate::connection::{Socket, SocketDirection};
use crate::errors::{GraphDoesNotExistSnafu, NodeDoesNotExistSnafu, NodeError, NodeOk, NodeResult};
use crate::node::NodeGetIoContext;
use crate::node_graph::NodeGraphDiff;
use crate::node_instance::NodeInstance;
use crate::nodes::variant_io;
use crate::state::ActionInvalidation;
use crate::{node::NodeIndex, node_graph::NodeGraph};

#[derive(Debug, Clone)]
pub enum DiffElement {
    GraphManagerDiff(GraphDiff<NodeGraph, ConnectedThrough>),
    ChildGraphDiff(GraphIndex, NodeGraphDiff),
    ExtendUiData(GlobalNodeIndex, SeaHashMap<String, Value>),
}

#[derive(Debug, Clone)]
pub struct GraphManagerDiff(pub Vec<DiffElement>);

impl GraphManagerDiff {
    pub fn from_graph_diffs(graph_index: GraphIndex, diffs: Vec<NodeGraphDiff>) -> GraphManagerDiff {
        GraphManagerDiff(
            diffs
                .into_iter()
                .map(|diff| DiffElement::ChildGraphDiff(graph_index, diff))
                .collect(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphIndex(pub VertexIndex);
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphEdgeIndex(pub EdgeIndex);
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectedThrough(pub NodeIndex);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalNodeIndex {
    pub graph_index: GraphIndex,
    pub node_index: NodeIndex,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphManager {
    node_graphs: Graph<NodeGraph, ConnectedThrough>,
    root_index: GraphIndex,
    #[serde(skip)]
    default_channel_count: usize,
}

impl GraphManager {
    pub fn new(default_channel_count: usize) -> Self {
        let mut graph = Graph::new();
        let (root_index, _) = graph.add_vertex(NodeGraph::new(default_channel_count));

        GraphManager {
            node_graphs: graph,
            root_index: GraphIndex(root_index),
            default_channel_count,
        }
    }

    pub fn new_graph(&mut self) -> Result<(GraphIndex, GraphManagerDiff), NodeError> {
        let (graph_index, add_diff) = self.node_graphs.add_vertex(NodeGraph::new(self.default_channel_count));

        let diff = GraphManagerDiff(vec![DiffElement::GraphManagerDiff(add_diff)]);

        Ok((GraphIndex(graph_index), diff))
    }

    pub fn connect_graphs(
        &mut self,
        from: GraphIndex,
        through: ConnectedThrough,
        to: GraphIndex,
    ) -> Result<(GraphEdgeIndex, GraphManagerDiff), NodeError> {
        let (edge_index, add_diff) = self.node_graphs.add_edge(from.0, to.0, through)?;

        let diff = GraphManagerDiff(vec![DiffElement::GraphManagerDiff(add_diff)]);

        Ok((GraphEdgeIndex(edge_index), diff))
    }

    pub fn disconnect_graphs(
        &mut self,
        from: GraphIndex,
        through: ConnectedThrough,
        to: GraphIndex,
    ) -> Result<(ConnectedThrough, GraphManagerDiff), NodeError> {
        let shared_edges: Vec<_> = self.node_graphs.shared_edges(from.0, to.0)?.collect();

        for edge_index in &shared_edges {
            let edge = self.node_graphs.get_edge(*edge_index).expect("edge to exist");

            if *edge.data() == through {
                let (old_data, remove_diff) = self.node_graphs.remove_edge(shared_edges[0])?;

                let diff = GraphManagerDiff(vec![DiffElement::GraphManagerDiff(remove_diff)]);

                return Ok((old_data, diff));
            }
        }

        Err(NodeError::GraphsNotConnected { from, through, to })
    }

    /// Will error out if there's more than one parent node connected
    pub fn remove_graph(&mut self, graph_index: GraphIndex) -> Result<(NodeGraph, GraphManagerDiff), NodeError> {
        let parent_nodes = self
            .node_graphs
            .get_vertex(graph_index.0)
            .with_context(|| GraphDoesNotExistSnafu { graph_index })?
            .get_connections_from();

        if !parent_nodes.is_empty() {
            Err(NodeError::GraphHasOtherParents)
        } else {
            let (old_data, remove_diff) = self.node_graphs.remove_vertex(graph_index.0)?;

            let diff = GraphManagerDiff(vec![DiffElement::GraphManagerDiff(remove_diff)]);

            Ok((old_data, diff))
        }
    }

    pub fn graphs(&self) -> impl Iterator<Item = GraphIndex> + '_ {
        self.node_graphs.vertex_indexes().map(GraphIndex)
    }

    pub fn root_index(&self) -> GraphIndex {
        self.root_index
    }

    pub fn get_graph(&self, index: GraphIndex) -> Result<&NodeGraph, NodeError> {
        Ok(self
            .node_graphs
            .get_vertex_data(index.0)
            .with_context(|| GraphDoesNotExistSnafu { graph_index: index })?)
    }

    pub fn get_graph_mut(&mut self, index: GraphIndex) -> Result<&mut NodeGraph, NodeError> {
        Ok(self
            .node_graphs
            .get_vertex_data_mut(index.0)
            .with_context(|| GraphDoesNotExistSnafu { graph_index: index })?)
    }

    pub fn set_default_channel_count(&mut self, default_channel_count: usize) {
        self.default_channel_count = default_channel_count;

        let indexes: Vec<_> = self.node_graphs.vertex_indexes().collect();

        for graph in indexes {
            let child_graph = self.node_graphs.get_vertex_data_mut(graph).unwrap();

            child_graph.set_default_channel_count(default_channel_count);
        }
    }

    pub fn get_node(&self, index: GlobalNodeIndex) -> Result<&NodeInstance, NodeError> {
        Ok(self.get_graph(index.graph_index)?.get_node(index.node_index)?)
    }

    pub fn get_node_mut(&mut self, index: GlobalNodeIndex) -> Result<&mut NodeInstance, NodeError> {
        Ok(self.get_graph_mut(index.graph_index)?.get_node_mut(index.node_index)?)
    }

    pub fn get_graph_parents(&self, graph_index: GraphIndex) -> Result<Vec<GlobalNodeIndex>, NodeError> {
        let parents = self
            .node_graphs
            .get_vertex(graph_index.0)
            .with_context(|| GraphDoesNotExistSnafu { graph_index })?
            .get_connections_from();

        let mapped = parents
            .iter()
            .map(|(vertex_index, edge_index)| {
                let edge = self.node_graphs.get_edge_data(*edge_index).expect("edge to exist");

                GlobalNodeIndex {
                    graph_index: GraphIndex(*vertex_index),
                    node_index: edge.0,
                }
            })
            .collect::<Vec<GlobalNodeIndex>>();

        Ok(mapped)
    }

    pub fn update_node(&mut self, index: GlobalNodeIndex, new: NodeInstance) -> Result<Vec<NodeGraphDiff>, NodeError> {
        let mut diffs = vec![];

        diffs.push(
            self.get_graph_mut(index.graph_index)?
                .update_node_no_row_updates(index.node_index, new)?,
        );
        diffs.extend(self.update_node_rows(index)?);

        Ok(diffs)
    }

    fn update_node_rows(&mut self, index: GlobalNodeIndex) -> Result<Vec<NodeGraphDiff>, NodeError> {
        let graph = self.get_graph(index.graph_index)?;
        let node = graph.get_node(index.node_index)?;

        let ctx = self.create_get_io_context(index)?;
        let new_rows = variant_io(&node.get_node_type(), &ctx, node.get_properties().clone())?.node_rows;

        let mut diffs = vec![];

        let graph = self.get_graph_mut(index.graph_index)?;
        let node = graph.get_node(index.node_index)?;

        // it would be pretty silly to do anything if the rows are exactly the same
        if *graph[index.node_index].get_node_rows() != new_rows {
            // Figure out what rows were removed.
            let removed: Vec<(Socket, SocketDirection)> = node
                .get_node_rows()
                .iter()
                .filter(|&old_row| !new_rows.iter().any(|new_row| new_row == old_row))
                .filter_map(|row| row.to_socket_and_direction().map(|x| (x.0.clone(), x.1)))
                .collect();

            // For any row that was removed, it needs to be disconnected (note we do all the disconnections
            // _before_ we set the new node rows, that way disconnecting doesn't error out and our invariants
            // stay all warm and fuzzy).
            for input_connection in graph.get_input_side_connections(index.node_index)? {
                if removed.iter().any(|(socket, direction)| {
                    socket == &input_connection.to_socket && direction == &SocketDirection::Input
                }) {
                    let (_, diff) = graph.disconnect(
                        input_connection.from_node,
                        &input_connection.from_socket,
                        index.node_index,
                        &input_connection.to_socket,
                    )?;

                    diffs.push(diff);
                }
            }

            for output_connection in graph.get_output_side_connections(index.node_index)? {
                if removed.iter().any(|(socket, direction)| {
                    socket == &output_connection.to_socket && direction == &SocketDirection::Output
                }) {
                    let (_, diff) = graph.disconnect(
                        index.node_index,
                        &output_connection.from_socket,
                        output_connection.to_node,
                        &output_connection.to_socket,
                    )?;

                    diffs.push(diff);
                }
            }

            let mut modified_node = graph.get_node(index.node_index)?.clone();
            modified_node.set_node_rows(new_rows);

            diffs.push(graph.update_node_no_row_updates(index.node_index, modified_node)?);
        }

        Ok(diffs)
    }

    fn create_get_io_context(&self, index: GlobalNodeIndex) -> Result<NodeGetIoContext, NodeError> {
        let graph = self.get_graph(index.graph_index)?.get_graph();
        let vertex = graph.get_vertex(index.node_index.0).context(NodeDoesNotExistSnafu {
            node_index: index.node_index,
        })?;

        let connected_inputs: Vec<Socket> = vertex
            .get_connections_from()
            .iter()
            .map(|(_, connection)| graph[*connection].data().to_socket.clone())
            .collect();
        let connected_outputs: Vec<Socket> = vertex
            .get_connections_to()
            .iter()
            .map(|(_, connection)| graph[*connection].data().from_socket.clone())
            .collect();

        Ok(NodeGetIoContext {
            default_channel_count: self.default_channel_count,
            connected_inputs,
            connected_outputs,
            child_graph: vertex
                .data()
                .get_child_graph()
                .map(|index| self.get_graph(index).unwrap()),
        })
    }
}

impl GraphManager {
    pub(crate) fn create_node(
        &mut self,
        node_type: &str,
        graph_index: GraphIndex,
        ui_data: SeaHashMap<String, Value>,
    ) -> NodeResult<(GraphManagerDiff, Vec<ActionInvalidation>)> {
        let mut diff: Vec<DiffElement> = vec![];

        let graph = self.get_graph_mut(graph_index)?;
        let creation_result = graph.add_node(node_type)?;

        let new_node_index = creation_result.value.0;

        diff.push(DiffElement::ChildGraphDiff(graph_index, creation_result.value.1));

        // does this node need a child graph?
        let new_node_instance = graph.get_node(new_node_index)?;
        let uses_child_graph = new_node_instance.uses_child_graph();

        let child_graph = if uses_child_graph {
            let new_graph_index = {
                // create a graph for it
                let (new_graph_index, creation_diff) = self.new_graph()?;
                diff.extend(creation_diff.0);

                // add `Inputs` node and `Outputs` node
                let new_graph = self.get_graph_mut(new_graph_index).expect("graph to have been created");

                let (inputs_index, inputs_diff) = new_graph.add_node("InputsNode".into())?.value;
                let (outputs_index, outputs_diff) = new_graph.add_node("OutputsNode".into())?.value;

                diff.push(DiffElement::ChildGraphDiff(new_graph_index, inputs_diff));
                diff.push(DiffElement::ChildGraphDiff(new_graph_index, outputs_diff));

                new_graph_index
            };

            // connect the two graphs together
            let (_, connect_diff) =
                self.connect_graphs(graph_index, ConnectedThrough(new_node_index), new_graph_index)?;
            diff.extend(connect_diff.0);

            Some(new_graph_index)
        } else {
            None
        };

        let new_node = self.get_graph_mut(graph_index)?.get_node_mut(new_node_index)?;

        if let Some(child_graph) = child_graph {
            new_node.set_child_graph(child_graph);
        }

        diff.push(DiffElement::ExtendUiData(
            GlobalNodeIndex {
                graph_index,
                node_index: new_node_index,
            },
            ui_data.clone(),
        ));

        new_node.extend_ui_data(ui_data);

        Ok(NodeOk::new(
            (
                GraphManagerDiff(diff),
                vec![
                    ActionInvalidation::NewNode(GlobalNodeIndex {
                        graph_index,
                        node_index: new_node_index,
                    }),
                    ActionInvalidation::GraphReindexNeeded(graph_index),
                ],
            ),
            creation_result.warnings,
        ))
    }

    pub(crate) fn connect_nodes(
        &mut self,
        graph: GraphIndex,
        from: NodeIndex,
        from_socket_type: &Socket,
        to: NodeIndex,
        to_socket_type: &Socket,
    ) -> Result<(GraphManagerDiff, ActionInvalidation), NodeError> {
        let (_, graph_diff) = self
            .get_graph_mut(graph)?
            .connect(from, from_socket_type, to, to_socket_type)?;

        let diff = GraphManagerDiff(vec![DiffElement::ChildGraphDiff(graph, graph_diff)]);

        Ok((diff, ActionInvalidation::GraphReindexNeeded(graph)))
    }

    pub(crate) fn disconnect_nodes(
        &mut self,
        graph: GraphIndex,
        from: NodeIndex,
        from_socket_type: &Socket,
        to: NodeIndex,
        to_socket_type: &Socket,
    ) -> Result<(GraphManagerDiff, ActionInvalidation), NodeError> {
        let (_, graph_diff) = self
            .get_graph_mut(graph)?
            .disconnect(from, from_socket_type, to, to_socket_type)?;

        let diff = GraphManagerDiff(vec![DiffElement::ChildGraphDiff(graph, graph_diff)]);

        Ok((diff, ActionInvalidation::GraphReindexNeeded(graph)))
    }

    pub(crate) fn remove_node(
        &mut self,
        index: GlobalNodeIndex,
    ) -> Result<(GraphManagerDiff, ActionInvalidation), NodeError> {
        let GlobalNodeIndex {
            graph_index,
            node_index,
        } = index;

        let mut diff = vec![];

        // first, see if the node has a child graph
        let children = self
            .node_graphs
            .get_vertex(graph_index.0)
            .with_context(|| GraphDoesNotExistSnafu {
                graph_index: graph_index,
            })?
            .get_connections_to();

        // if it does have children, remove the connections
        if !children.is_empty() {
            let (_, remove_diff) =
                self.disconnect_graphs(graph_index, ConnectedThrough(node_index), GraphIndex(children[0].0))?;
            diff.extend(remove_diff.0);
        }

        // now, we can remove the node
        let (_, remove_diff) = self.get_graph_mut(graph_index)?.remove_node(node_index)?;
        diff.push(DiffElement::ChildGraphDiff(graph_index, remove_diff));

        Ok((
            GraphManagerDiff(diff),
            ActionInvalidation::GraphReindexNeeded(graph_index),
        ))
    }

    pub(crate) fn reapply_action(&mut self, diff: GraphManagerDiff) -> Result<Vec<ActionInvalidation>, NodeError> {
        let mut invalidations = vec![];

        for part in diff.0 {
            match part {
                DiffElement::GraphManagerDiff(diff) => self.node_graphs.apply_diff(diff)?,
                DiffElement::ChildGraphDiff(graph_index, diff) => {
                    let new_invalidation = ActionInvalidation::GraphReindexNeeded(graph_index);

                    if !invalidations.iter().any(|inv| inv == &new_invalidation) {
                        invalidations.push(new_invalidation);
                    }

                    self.get_graph_mut(graph_index)?.apply_diff(diff)?
                }
                DiffElement::ExtendUiData(index, ui_data) => {
                    self.get_node_mut(index)?.extend_ui_data(ui_data.clone());
                }
            };
        }

        Ok(invalidations)
    }

    pub(crate) fn rollback_action(&mut self, diff: GraphManagerDiff) -> Result<Vec<ActionInvalidation>, NodeError> {
        let mut invalidations = vec![];

        for part in diff.0 {
            match part {
                DiffElement::GraphManagerDiff(diff) => self.node_graphs.rollback_diff(diff)?,
                DiffElement::ChildGraphDiff(graph_index, diff) => {
                    let new_invalidation = ActionInvalidation::GraphReindexNeeded(graph_index);

                    if !invalidations.iter().any(|inv| inv == &new_invalidation) {
                        invalidations.push(new_invalidation);
                    }

                    self.get_graph_mut(graph_index)?.rollback_diff(diff)?
                }
                DiffElement::ExtendUiData(..) => {}
            }
        }

        Ok(invalidations)
    }
}
