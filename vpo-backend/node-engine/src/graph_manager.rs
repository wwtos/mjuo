use std::collections::HashMap;

use ddgg::{EdgeIndex, Graph, GraphDiff, VertexIndex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::OptionExt;

use crate::connection::Socket;
use crate::errors::{GraphDoesNotExistSnafu, NodeError, NodeOk, NodeResult};
use crate::node_graph::NodeGraphDiff;
use crate::node_instance::NodeInstance;
use crate::socket_registry::SocketRegistry;
use crate::state::ActionInvalidation;
use crate::{node::NodeIndex, node_graph::NodeGraph};

#[derive(Debug, Clone)]
pub enum DiffElement {
    GraphManagerDiff(GraphDiff<NodeGraph, ConnectedThrough>),
    ChildGraphDiff(GraphIndex, NodeGraphDiff),
    ExtendUiData(GlobalNodeIndex, HashMap<String, Value>),
}

#[derive(Debug, Clone)]
pub struct GraphManagerDiff(pub Vec<DiffElement>);

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
}

impl GraphManager {
    pub fn new() -> Self {
        let mut graph = Graph::new();
        let (root_index, _) = graph.add_vertex(NodeGraph::new()).unwrap();

        GraphManager {
            node_graphs: graph,
            root_index: GraphIndex(root_index),
        }
    }

    pub fn new_graph(&mut self) -> Result<(GraphIndex, GraphManagerDiff), NodeError> {
        let (graph_index, add_diff) = self.node_graphs.add_vertex(NodeGraph::new())?;

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
        let shared_edges = self.node_graphs.shared_edges(from.0, to.0)?;

        for edge_index in &shared_edges {
            let edge = self.node_graphs.get_edge(*edge_index).expect("edge to exist");

            if edge.data == through {
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
}

impl GraphManager {
    pub(crate) fn create_node(
        &mut self,
        node_type: &str,
        graph_index: GraphIndex,
        registry: &mut SocketRegistry,
        ui_data: HashMap<String, Value>,
    ) -> NodeResult<(GraphManagerDiff, Vec<ActionInvalidation>)> {
        let mut diff = vec![];

        let graph = self.get_graph_mut(graph_index)?;
        let creation_result = graph.add_node(node_type.into(), registry)?;

        let new_node_index = creation_result.value.0;

        diff.push(DiffElement::ChildGraphDiff(graph_index, creation_result.value.1));

        // does this node need a child graph?
        let new_node_instance = graph.get_node(new_node_index)?;
        let uses_child_graph = new_node_instance.uses_child_graph();

        let child_graph_index = if uses_child_graph {
            let new_graph_index = {
                // create a graph for it
                let (new_graph_index, creation_diff) = self.new_graph()?;
                diff.extend(creation_diff.0);

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

        if let Some(child_graph_index) = child_graph_index {
            new_node.set_child_graph_index(child_graph_index);
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
        from_socket_type: Socket,
        to: NodeIndex,
        to_socket_type: Socket,
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
        from_socket_type: Socket,
        to: NodeIndex,
        to_socket_type: Socket,
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
