use std::cell::RefCell;

use ddgg::{EdgeIndex, Graph, GraphDiff, GraphError, VertexIndex};
use serde::{Deserialize, Serialize};

use crate::connection::Socket;
use crate::errors::{NodeError, NodeOk, NodeResult, WarningBuilder};
use crate::node_graph::NodeGraphDiff;
use crate::socket_registry::SocketRegistry;
use crate::state::ActionInvalidations;
use crate::{node::NodeIndex, node_graph::NodeGraph};

#[derive(Debug, Clone)]
enum DiffElement {
    GraphManagerDiff(GraphDiff<NodeGraphWrapper, ConnectedThrough>),
    ChildGraphDiff(GraphIndex, NodeGraphDiff),
}

#[derive(Debug, Clone)]
pub struct GraphManagerDiff(Vec<DiffElement>);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeGraphWrapper {
    pub graph: RefCell<NodeGraph>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphManager {
    node_graphs: Graph<NodeGraphWrapper, ConnectedThrough>,
    root_index: GraphIndex,
}

impl GraphManager {
    pub fn new() -> Self {
        let mut graph = Graph::new();
        let (root_index, _) = graph
            .add_vertex(NodeGraphWrapper {
                graph: RefCell::new(NodeGraph::new()),
            })
            .unwrap();

        GraphManager {
            node_graphs: graph,
            root_index: GraphIndex(root_index),
        }
    }

    pub fn new_graph(&mut self) -> Result<(GraphIndex, GraphManagerDiff), NodeError> {
        let (graph_index, add_diff) = self.node_graphs.add_vertex(NodeGraphWrapper {
            graph: RefCell::new(NodeGraph::new()),
        })?;

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
            let edge = self.node_graphs.get_edge(*edge_index)?;

            if edge.data == through {
                let (old_data, remove_diff) = self.node_graphs.remove_edge(shared_edges[0])?;

                let diff = GraphManagerDiff(vec![DiffElement::GraphManagerDiff(remove_diff)]);

                return Ok((old_data, diff));
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
            let (old_data, remove_diff) = self.node_graphs.remove_vertex(graph_index.0)?;

            let diff = GraphManagerDiff(vec![DiffElement::GraphManagerDiff(remove_diff)]);

            return Ok((old_data, diff));
        }
    }

    pub fn root_index(&self) -> GraphIndex {
        self.root_index
    }

    pub fn get_graph(&self, index: GraphIndex) -> Result<&NodeGraphWrapper, NodeError> {
        Ok(self.node_graphs.get_vertex_data(index.0)?)
    }

    pub fn get_graph_mut(&mut self, index: GraphIndex) -> Result<&mut NodeGraphWrapper, NodeError> {
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
}

impl GraphManager {
    pub fn create_node(
        &mut self,
        node_type: &str,
        graph_index: GraphIndex,
        registry: &mut SocketRegistry,
    ) -> NodeResult<(GraphManagerDiff, ActionInvalidations)> {
        let mut warnings = WarningBuilder::new();

        let mut diff = vec![];
        let mut graph = self.get_graph(graph_index)?.graph.borrow_mut();
        let creation_result = graph.add_node(node_type.into(), registry)?;

        let new_node_index = creation_result.value.0;
        warnings.append_warnings(creation_result.warnings);

        diff.push(DiffElement::ChildGraphDiff(graph_index, creation_result.value.1));

        // does this node need a child graph?
        let new_node_wrapper = graph.get_node(new_node_index)?;
        let uses_child_graph = new_node_wrapper.uses_child_graph();

        drop(graph);

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

        let graph = &mut self.get_graph(graph_index)?.graph.borrow_mut();
        let new_node = graph.get_node_mut(new_node_index)?;

        if let Some(child_graph_index) = child_graph_index {
            new_node.set_child_graph_index(child_graph_index);
        }

        let invalidations = ActionInvalidations {
            graph_to_reindex: None,
            graph_operated_on: Some(graph_index),
            defaults_to_update: None,
            nodes_created: vec![GlobalNodeIndex {
                node_index: new_node_index,
                graph_index,
            }],
        };

        Ok(NodeOk::new(
            (GraphManagerDiff(diff), invalidations),
            warnings.into_warnings(),
        ))
    }

    pub fn connect_nodes(
        &mut self,
        from: GlobalNodeIndex,
        from_socket_type: Socket,
        to: GlobalNodeIndex,
        to_socket_type: Socket,
    ) -> Result<(GraphManagerDiff, ActionInvalidations), NodeError> {
        if from.graph_index != to.graph_index {
            return Err(NodeError::MismatchedNodeGraphs { from, to });
        }

        let mut graph = self.get_graph(from.graph_index)?.graph.borrow_mut();

        let (_, graph_diff) = graph.connect(from.node_index, from_socket_type, to.node_index, to_socket_type)?;

        let invalidations = ActionInvalidations {
            graph_to_reindex: Some(from.graph_index),
            graph_operated_on: Some(from.graph_index),
            defaults_to_update: None,
            nodes_created: vec![],
        };

        let diff = GraphManagerDiff(vec![DiffElement::ChildGraphDiff(from.graph_index, graph_diff)]);

        Ok((diff, invalidations))
    }

    pub fn disconnect_nodes(
        &mut self,
        from: GlobalNodeIndex,
        from_socket_type: Socket,
        to: GlobalNodeIndex,
        to_socket_type: Socket,
    ) -> Result<(GraphManagerDiff, ActionInvalidations), NodeError> {
        if from.graph_index != to.graph_index {
            return Err(NodeError::MismatchedNodeGraphs { from, to });
        }

        let mut graph = self.get_graph(from.graph_index)?.graph.borrow_mut();

        let (_, graph_diff) = graph.disconnect(from.node_index, from_socket_type, to.node_index, to_socket_type)?;

        let invalidations = ActionInvalidations {
            graph_to_reindex: Some(from.graph_index),
            graph_operated_on: Some(from.graph_index),
            defaults_to_update: None,
            nodes_created: vec![],
        };

        let diff = GraphManagerDiff(vec![DiffElement::ChildGraphDiff(from.graph_index, graph_diff)]);

        Ok((diff, invalidations))
    }

    pub fn remove_node(
        &mut self,
        index: GlobalNodeIndex,
    ) -> Result<(GraphManagerDiff, ActionInvalidations), NodeError> {
        let GlobalNodeIndex {
            graph_index,
            node_index,
        } = index;

        let mut diff = vec![];

        // first, see if the node has a child graph
        let children = self.node_graphs.get_vertex(graph_index.0)?.get_connections_to();

        // if it does have children, remove the connections
        if !children.is_empty() {
            let (_, remove_diff) =
                self.disconnect_graphs(graph_index, ConnectedThrough(node_index), GraphIndex(children[0].0))?;
            diff.extend(remove_diff.0);
        }

        // now that we've ensured that the graph is disconnected, we can safely delete the node
        // but first, we need to make a list of its current connections (for undo)
        let mut graph = self.get_graph(graph_index)?.graph.borrow_mut();

        // now, we can remove the node
        let (_, remove_diff) = graph.remove_node(node_index)?;
        diff.push(DiffElement::ChildGraphDiff(graph_index, remove_diff));

        let invalidations = ActionInvalidations {
            graph_to_reindex: Some(graph_index),
            graph_operated_on: Some(graph_index),
            defaults_to_update: None,
            nodes_created: vec![],
        };

        Ok((GraphManagerDiff(diff), invalidations))
    }

    pub fn reapply_action(&mut self, diff: GraphManagerDiff) -> Result<ActionInvalidations, NodeError> {
        let mut invalidations = ActionInvalidations {
            graph_to_reindex: None,
            graph_operated_on: None,
            defaults_to_update: None,
            nodes_created: vec![],
        };

        for part in diff.0 {
            match part {
                DiffElement::GraphManagerDiff(diff) => self.node_graphs.apply_diff(diff)?,
                DiffElement::ChildGraphDiff(graph_index, diff) => {
                    invalidations.graph_to_reindex = Some(graph_index);
                    invalidations.graph_operated_on = Some(graph_index);

                    match &diff {
                        GraphDiff::AddVertex(diff) => invalidations.nodes_created.push(GlobalNodeIndex {
                            node_index: NodeIndex(diff.get_vertex_index()),
                            graph_index,
                        }),
                        _ => {}
                    }

                    self.get_graph(graph_index)?.graph.borrow_mut().apply_diff(diff)?
                }
            }
        }

        Ok(invalidations)
    }

    pub fn rollback_action(&mut self, diff: GraphManagerDiff) -> Result<ActionInvalidations, NodeError> {
        let mut invalidations = ActionInvalidations {
            graph_to_reindex: None,
            graph_operated_on: None,
            defaults_to_update: None,
            nodes_created: vec![],
        };

        for part in diff.0 {
            match part {
                DiffElement::GraphManagerDiff(diff) => self.node_graphs.rollback_diff(diff)?,
                DiffElement::ChildGraphDiff(graph_index, diff) => {
                    invalidations.graph_to_reindex = Some(graph_index);
                    invalidations.graph_operated_on = Some(graph_index);

                    self.get_graph(graph_index)?.graph.borrow_mut().rollback_diff(diff)?
                }
            }
        }

        Ok(invalidations)
    }
}
