use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    connection::{Primitive, Socket, SocketType, SocketValue},
    engine::NodeEngine,
    errors::{NodeError, WarningExt},
    global_state::{GlobalState, Resources},
    graph_manager::{GlobalNodeIndex, GraphIndex, GraphManager, GraphManagerDiff},
    node::{NodeGetIoContext, NodeIndex, NodeRow, NodeState},
    node_graph::{NodeConnectionData, NodeGraph},
    nodes::variant_io,
    property::Property,
    traversal::buffered_traverser::BufferedTraverser,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoNodes {
    pub input: NodeIndex,
    pub output: NodeIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum Action {
    CreateNode {
        graph: GraphIndex,
        #[serde(rename = "nodeType")]
        node_type: String,
        #[serde(rename = "uiData")]
        ui_data: HashMap<String, Value>,
    },
    ConnectNodes {
        graph: GraphIndex,
        from: NodeIndex,
        to: NodeIndex,
        data: NodeConnectionData,
    },
    DisconnectNodes {
        graph: GraphIndex,
        from: NodeIndex,
        to: NodeIndex,
        data: NodeConnectionData,
    },
    RemoveNode {
        index: GlobalNodeIndex,
    },
    ChangeNodeProperties {
        index: GlobalNodeIndex,
        props: HashMap<String, Property>,
    },
    ChangeNodeUiData {
        index: GlobalNodeIndex,
        #[serde(rename = "uiData")]
        ui_data: HashMap<String, Value>,
    },
    ChangeNodeOverrides {
        index: GlobalNodeIndex,
        overrides: Vec<NodeRow>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionBundle {
    pub actions: Vec<Action>,
}

impl ActionBundle {
    pub fn new(actions: Vec<Action>) -> ActionBundle {
        ActionBundle { actions }
    }
}

#[derive(PartialEq)]
pub enum ActionInvalidation {
    GraphReindexNeeded(GraphIndex),
    GraphModified(GraphIndex),
    NewDefaults(GlobalNodeIndex, Vec<(Socket, SocketValue)>),
    NewNode(GlobalNodeIndex),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionCategory {
    Separate,
    Mergable,
}

#[derive(Debug)]
pub enum NodeEngineUpdate {
    NewNodeEngine(NodeEngine),
    NewDefaults(Vec<(NodeIndex, Socket, Primitive)>),
    NewNodeState(Vec<(NodeIndex, serde_json::Value)>),
    CurrentNodeStates(BTreeMap<NodeIndex, NodeState>),
}

#[derive(Debug, Clone)]
pub enum FromNodeEngine {
    NodeStateUpdates(Vec<(NodeIndex, NodeState)>),
    RequestedStateUpdates(Vec<(NodeIndex, serde_json::Value)>),
    GraphStateRequested,
}

#[derive(Clone, Debug)]
pub struct HistoryAction {
    diff: GraphManagerDiff,
    category: ActionCategory,
}

#[derive(Clone, Debug)]
pub struct HistoryActionBundle {
    pub actions: Vec<HistoryAction>,
}

#[derive(Debug)]
pub struct GraphState {
    history: Vec<HistoryActionBundle>,
    place_in_history: usize,
    graph_manager: GraphManager,
    root_graph_index: GraphIndex,
    io_nodes: IoNodes,
    default_channel_count: usize,
}

impl GraphState {
    pub fn new(global_state: &GlobalState) -> Result<GraphState, NodeError> {
        let default_channel_count = global_state.default_channel_count;

        let history = Vec::new();
        let place_in_history = 0;

        let mut graph_manager: GraphManager = GraphManager::new(default_channel_count);

        let root_graph_index = graph_manager.root_index();

        let (output_node, input_node) = {
            let graph = graph_manager.get_graph_mut(root_graph_index)?;

            let (output_node, _) = graph.add_node("OutputsNode".into())?.value;
            let (input_node, _) = graph.add_node("InputsNode".into())?.value;

            let mut modified_input = graph.get_node(input_node)?.clone();
            let mut modified_output = graph.get_node(output_node)?.clone();

            modified_input.set_property(
                "socket_list".into(),
                Property::SocketList(vec![Socket::Simple("midi".into(), SocketType::Midi, 1)]),
            );
            modified_output.set_property(
                "socket_list".into(),
                Property::SocketList(vec![Socket::Simple("audio".into(), SocketType::Stream, 1)]),
            );

            graph.update_node(input_node, modified_input)?;
            graph.update_node(output_node, modified_output)?;

            (output_node, input_node)
        };

        Ok(GraphState {
            history,
            place_in_history,
            graph_manager,
            root_graph_index,
            io_nodes: IoNodes {
                input: input_node,
                output: output_node,
            },
            default_channel_count: global_state.default_channel_count,
        })
    }

    pub fn get_traverser(
        &self,
        global_state: &GlobalState,
        resources: &Resources,
    ) -> Result<BufferedTraverser, NodeError> {
        let traverser = BufferedTraverser::new(
            global_state.sound_config.clone(),
            &self.graph_manager,
            self.root_graph_index,
            &resources,
            0,
        )?;

        Ok(traverser)
    }

    pub fn get_engine(&self, global_state: &GlobalState, resources: &Resources) -> Result<NodeEngine, NodeError> {
        let traverser = BufferedTraverser::new(
            global_state.sound_config.clone(),
            &self.graph_manager,
            self.root_graph_index,
            &resources,
            0,
        )?;

        Ok(NodeEngine::new(traverser, self.io_nodes.clone()))
    }

    pub fn get_node_state(&self) -> BTreeMap<NodeIndex, NodeState> {
        let root = self
            .graph_manager
            .get_graph(self.root_graph_index)
            .expect("root graph to exist");

        let mut result = BTreeMap::new();

        for (index, node) in root.nodes_data_iter() {
            if !matches!(node.get_state().value, Value::Null) {
                result.insert(index, node.get_state().clone());
            }
        }

        result
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.place_in_history = 0;
    }

    pub fn get_graph_manager(&mut self) -> &mut GraphManager {
        &mut self.graph_manager
    }

    pub fn get_root_graph_index(&self) -> GraphIndex {
        self.root_graph_index
    }

    pub fn get_root_graph(&self) -> &NodeGraph {
        self.graph_manager
            .get_graph(self.root_graph_index)
            .expect("root graph to exist")
    }

    pub fn get_io_nodes(&self) -> IoNodes {
        self.io_nodes.clone()
    }

    pub fn notify_parents_of_graph_change(&mut self, graph_index: GraphIndex) -> Result<(), NodeError> {
        if graph_index != self.graph_manager.root_index() {
            let parent_nodes = self.graph_manager.get_graph_parents(graph_index)?;

            for GlobalNodeIndex {
                graph_index: parent_node_graph,
                node_index: parent_node_index,
            } in parent_nodes
            {
                let parent_node_graph = self.graph_manager.get_graph(parent_node_graph)?;
                // let subgraph = &mut self.graph_manager.get_graph(graph_index)?.graph.borrow_mut();

                // let node = parent_node_graph.get_node_mut(parent_node_index)?;
            }
        }

        Ok(())
    }
}

impl GraphState {
    pub fn get_history(&self) -> &Vec<HistoryActionBundle> {
        &self.history
    }

    pub fn invalidations_to_engine_updates(
        &self,
        invalidations: Vec<ActionInvalidation>,
        global_state: &GlobalState,
        resources: &Resources,
    ) -> Result<Vec<NodeEngineUpdate>, NodeError> {
        let mut root_graph_reindex_needed = false;
        let mut new_defaults = vec![];

        for invalidation in invalidations {
            match invalidation {
                ActionInvalidation::GraphReindexNeeded(index) => {
                    if index == self.root_graph_index {
                        root_graph_reindex_needed = true;
                    }
                }
                ActionInvalidation::NewDefaults(index, defaults) => {
                    if index.graph_index == self.root_graph_index {
                        new_defaults.extend(defaults.into_iter().filter_map(|(socket, value)| {
                            if let Some(value) = value.as_value() {
                                Some((index.node_index, socket, value))
                            } else {
                                None
                            }
                        }))
                    }
                }
                ActionInvalidation::None => {}
                ActionInvalidation::NewNode(_) => {}
                ActionInvalidation::GraphModified(_) => {}
            }
        }

        let mut updates = vec![];

        if root_graph_reindex_needed {
            updates.push(NodeEngineUpdate::NewNodeEngine(
                self.get_engine(global_state, resources)?,
            ));
        }

        if !new_defaults.is_empty() {
            updates.push(NodeEngineUpdate::NewDefaults(new_defaults));
        }

        Ok(updates)
    }

    pub fn commit(&mut self, actions: ActionBundle, force_append: bool) -> Result<Vec<ActionInvalidation>, NodeError> {
        let (mut new_actions, action_results) = actions
            .actions
            .into_iter()
            .map(|action| self.apply_action(action))
            .collect::<Result<Vec<(HistoryAction, Vec<ActionInvalidation>)>, NodeError>>()?
            .into_iter()
            .unzip::<HistoryAction, Vec<ActionInvalidation>, Vec<HistoryAction>, Vec<Vec<ActionInvalidation>>>();

        if self.place_in_history < self.history.len() {
            self.history.truncate(self.place_in_history);
        }

        // determine whether to add a new action bundle, or to concatinate it to the current
        // action bundle
        if !self.history.is_empty() {
            let is_new_bundle_mergable = new_actions.iter().all(|x| x.category == ActionCategory::Mergable);
            let is_current_bundle_mergable = self.history[self.place_in_history - 1]
                .actions
                .iter()
                .all(|x| x.category == ActionCategory::Mergable);

            let should_append = force_append || (is_current_bundle_mergable && is_new_bundle_mergable);

            if should_append {
                self.history[self.place_in_history - 1].actions.append(&mut new_actions);
            } else {
                self.history.push(HistoryActionBundle { actions: new_actions });

                self.place_in_history += 1;
            }
        } else {
            self.history.push(HistoryActionBundle { actions: new_actions });

            self.place_in_history += 1;
        }

        let invalidations = action_results.into_iter().flatten().collect();

        Ok(invalidations)
    }

    pub fn undo(&mut self) -> Result<Vec<ActionInvalidation>, NodeError> {
        if self.place_in_history > 0 {
            let to_rollback = self.history[self.place_in_history - 1].clone();

            // roll back in reverse order
            let invalidations = to_rollback
                .actions
                .into_iter()
                .rev()
                .map(|action| self.rollback_action(action))
                .flatten_ok()
                .collect::<Result<Vec<ActionInvalidation>, NodeError>>()?;

            self.place_in_history -= 1;

            Ok(invalidations)
        } else {
            Ok(vec![])
        }
    }

    pub fn redo(&mut self) -> Result<Vec<ActionInvalidation>, NodeError> {
        if self.place_in_history < self.history.len() {
            let to_redo = self.history[self.place_in_history].clone();

            let invalidations = to_redo
                .actions
                .into_iter()
                .map(|action| self.reapply_action(action))
                .flatten_ok()
                .collect::<Result<Vec<ActionInvalidation>, NodeError>>()?;

            self.place_in_history += 1;

            Ok(invalidations)
        } else {
            Ok(vec![])
        }
    }

    fn apply_action(&mut self, action: Action) -> Result<(HistoryAction, Vec<ActionInvalidation>), NodeError> {
        let mut warnings = vec![];

        let new_action = match action {
            Action::CreateNode {
                graph: graph_index,
                node_type,
                ui_data,
            } => {
                let (diff, invalidations) = self
                    .graph_manager
                    .create_node(&node_type, graph_index, ui_data.clone())
                    .append_warnings(&mut warnings)?;

                (
                    HistoryAction {
                        diff,
                        category: ActionCategory::Separate,
                    },
                    invalidations,
                )
            }
            Action::ConnectNodes { graph, from, to, data } => {
                let (diff, invalidations) =
                    self.graph_manager
                        .connect_nodes(graph, from, &data.from_socket, to, &data.to_socket)?;

                (
                    HistoryAction {
                        diff,
                        category: ActionCategory::Separate,
                    },
                    vec![invalidations],
                )
            }
            Action::DisconnectNodes { graph, from, to, data } => {
                let (diff, invalidations) =
                    self.graph_manager
                        .disconnect_nodes(graph, from, &data.from_socket, to, &data.to_socket)?;

                (
                    HistoryAction {
                        diff,
                        category: ActionCategory::Separate,
                    },
                    vec![invalidations],
                )
            }
            Action::RemoveNode { index } => {
                if index.graph_index == self.root_graph_index
                    && (index.node_index == self.io_nodes.input || index.node_index == self.io_nodes.output)
                {
                    return Err(NodeError::CannotDeleteRootNode);
                }

                let (diff, invalidation) = self.graph_manager.remove_node(index)?;

                (
                    HistoryAction {
                        diff,
                        category: ActionCategory::Separate,
                    },
                    vec![invalidation],
                )
            }
            Action::ChangeNodeProperties {
                index,
                props: new_props,
            } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let mut modified_node = graph.get_node(index.node_index)?.clone();

                modified_node.set_properties(new_props.clone());

                let diffs = graph.update_node(index.node_index, modified_node)?;

                (
                    HistoryAction {
                        diff: GraphManagerDiff::from_graph_diffs(index.graph_index, diffs),
                        category: ActionCategory::Mergable,
                    },
                    vec![ActionInvalidation::GraphReindexNeeded(index.graph_index)],
                )
            }
            Action::ChangeNodeUiData { index, ui_data: data } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let mut modified_node = graph.get_node(index.node_index)?.clone();

                modified_node.set_ui_data(data.clone());

                let diffs = graph.update_node(index.node_index, modified_node)?;

                (
                    HistoryAction {
                        diff: GraphManagerDiff::from_graph_diffs(index.graph_index, diffs),
                        category: ActionCategory::Mergable,
                    },
                    vec![ActionInvalidation::GraphReindexNeeded(index.graph_index)],
                )
            }
            Action::ChangeNodeOverrides { index, overrides } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let mut modified_node = graph.get_node(index.node_index)?.clone();

                modified_node.set_default_overrides(overrides.clone());

                let diffs = graph.update_node(index.node_index, modified_node)?;

                (
                    HistoryAction {
                        diff: GraphManagerDiff::from_graph_diffs(index.graph_index, diffs),
                        category: ActionCategory::Mergable,
                    },
                    vec![ActionInvalidation::GraphReindexNeeded(index.graph_index)],
                )
            }
        };

        Ok(new_action)
    }

    fn reapply_action(&mut self, action: HistoryAction) -> Result<Vec<ActionInvalidation>, NodeError> {
        let invalidations = self.graph_manager.reapply_action(action.diff)?;

        Ok(invalidations)
    }

    fn rollback_action(&mut self, action: HistoryAction) -> Result<Vec<ActionInvalidation>, NodeError> {
        let invalidations = self.graph_manager.rollback_action(action.diff)?;

        Ok(invalidations)
    }
}

impl GraphState {
    pub fn to_json(&self) -> Value {
        json!({
            "graphManager": self.graph_manager,
            "rootGraphIndex": self.root_graph_index,
            "ioNodes": self.io_nodes,
        })
    }

    pub fn load_state(&mut self, graph_manager: GraphManager, root_graph_index: GraphIndex, io_nodes: IoNodes) {
        self.history.clear();
        self.place_in_history = 0;
        self.graph_manager = graph_manager;
        self.root_graph_index = root_graph_index;
        self.io_nodes = io_nodes;

        let graphs: Vec<GraphIndex> = self.graph_manager.graphs().collect();
        for graph_index in graphs {
            let graph = self
                .graph_manager
                .get_graph_mut(graph_index)
                .expect("graph_index to exist");

            let nodes: Vec<NodeIndex> = graph.node_indexes().collect();

            for node_index in nodes {
                let node = graph.get_node_mut(node_index).expect("node_index to exist");

                node.set_node_rows(
                    variant_io(
                        &node.get_node_type(),
                        &NodeGetIoContext::no_io_yet(self.default_channel_count),
                        node.get_properties().clone(),
                    )
                    .unwrap()
                    .node_rows,
                );
            }
        }
    }
}
