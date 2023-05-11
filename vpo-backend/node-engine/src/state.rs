use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    connection::{Primitive, Socket, SocketValue},
    engine::NodeEngine,
    errors::{NodeError, WarningExt},
    global_state::GlobalState,
    graph_manager::{DiffElement, GlobalNodeIndex, GraphIndex, GraphManager, GraphManagerDiff},
    node::{NodeIndex, NodeRow},
    node_graph::NodeConnection,
    property::Property,
    socket_registry::SocketRegistry,
    traversal::buffered_traverser::BufferedTraverser,
};
use rhai::Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoNodes {
    pub input: NodeIndex,
    pub output: NodeIndex,
}

#[derive(Debug, Clone)]
pub enum Action {
    AddNode {
        graph: GraphIndex,
        node_type: String,
    },
    ConnectNodes {
        from: GlobalNodeIndex,
        to: GlobalNodeIndex,
        data: NodeConnection,
    },
    DisconnectNodes {
        from: GlobalNodeIndex,
        to: GlobalNodeIndex,
        data: NodeConnection,
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
        data: HashMap<String, Value>,
    },
    ChangeNodeOverrides {
        index: GlobalNodeIndex,
        overrides: Vec<NodeRow>,
    },
}

#[derive(Clone, Debug)]
pub struct ActionBundle {
    actions: Vec<Action>,
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

#[derive(Debug)]
pub enum NodeEngineUpdate {
    NewNodeEngine(NodeEngine),
    NewDefaults(Vec<(NodeIndex, Socket, Primitive)>),
    NewUiState(Vec<(NodeIndex, serde_json::Value)>),
}

#[derive(Debug, Clone)]
pub enum FromNodeEngine {
    UiUpdates(Vec<(NodeIndex, serde_json::Value)>),
}

#[derive(Clone, Debug)]
pub enum HistoryAction {
    GraphAction {
        diff: GraphManagerDiff,
    },
    ChangeNodeProperties {
        index: GlobalNodeIndex,
        before: HashMap<String, Property>,
        after: HashMap<String, Property>,
        graph_diff: GraphManagerDiff,
    },
    ChangeNodeUiData {
        index: GlobalNodeIndex,
        before: HashMap<String, Value>,
        after: HashMap<String, Value>,
    },
    ChangeNodeOverrides {
        index: GlobalNodeIndex,
        before: Vec<NodeRow>,
        after: Vec<NodeRow>,
    },
}

#[derive(Clone, Debug)]
pub struct HistoryActionBundle {
    pub actions: Vec<HistoryAction>,
}

#[derive(Debug)]
pub struct NodeState {
    history: Vec<HistoryActionBundle>,
    place_in_history: usize,
    graph_manager: GraphManager,
    root_graph_index: GraphIndex,
    io_nodes: IoNodes,
    socket_registry: SocketRegistry,
}

impl NodeState {
    pub fn new(global_state: &GlobalState) -> Result<NodeState, NodeError> {
        let history = Vec::new();
        let place_in_history = 0;

        let mut graph_manager: GraphManager = GraphManager::new();
        let mut socket_registry = SocketRegistry::default();

        let root_graph_index = graph_manager.root_index();

        let (output_node, input_node) = {
            let graph = graph_manager.get_graph_mut(root_graph_index)?;

            let output_node = graph.add_node("OutputNode".into(), &mut socket_registry).unwrap().value;
            let input_node = graph.add_node("MidiInNode".into(), &mut socket_registry).unwrap().value;

            (output_node.0, input_node.0)
        };

        let scripting_engine: Engine = Engine::new_raw();
        let mut root_traverser = BufferedTraverser::default();

        root_traverser.init_graph(
            root_graph_index,
            &graph_manager,
            &scripting_engine,
            &global_state.resources.read().unwrap(),
            0,
            global_state.sound_config.clone(),
        )?;

        Ok(NodeState {
            history,
            place_in_history,
            graph_manager,
            root_graph_index,
            io_nodes: IoNodes {
                input: input_node,
                output: output_node,
            },
            socket_registry,
        })
    }

    pub fn get_traverser(&self, global_state: &GlobalState) -> Result<BufferedTraverser, NodeError> {
        let script_engine = rhai::Engine::new_raw();
        let resources = global_state.resources.read().unwrap();

        let (traverser, errors_and_warnings) = BufferedTraverser::new(
            self.root_graph_index,
            &self.graph_manager,
            &script_engine,
            &resources,
            0,
            global_state.sound_config.clone(),
        )?;

        Ok(traverser)
    }

    pub fn get_engine(&self, global_state: &GlobalState) -> Result<NodeEngine, NodeError> {
        let script_engine = rhai::Engine::new_raw();
        let resources = global_state.resources.read().unwrap();

        let (traverser, errors_and_warnings) = BufferedTraverser::new(
            self.root_graph_index,
            &self.graph_manager,
            &script_engine,
            &resources,
            0,
            global_state.sound_config.clone(),
        )?;

        Ok(NodeEngine::new(traverser, script_engine, self.io_nodes.clone()))
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

    pub fn get_registry(&self) -> &SocketRegistry {
        &self.socket_registry
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

impl NodeState {
    pub fn get_history(&self) -> &Vec<HistoryActionBundle> {
        &self.history
    }

    fn is_action_property_related(action: &HistoryAction) -> bool {
        matches!(
            action,
            HistoryAction::ChangeNodeProperties { .. }
                | HistoryAction::ChangeNodeUiData { .. }
                | HistoryAction::ChangeNodeOverrides { .. }
        )
    }

    pub fn invalidations_to_engine_updates(
        &self,
        invalidations: Vec<ActionInvalidation>,
        global_state: &GlobalState,
    ) -> Vec<NodeEngineUpdate> {
        invalidations
            .into_iter()
            .filter_map(|inv| match inv {
                ActionInvalidation::GraphReindexNeeded(index) => {
                    if index == self.root_graph_index {
                        Some(NodeEngineUpdate::NewNodeEngine(self.get_engine(global_state).ok()?))
                    } else {
                        None
                    }
                }
                ActionInvalidation::NewDefaults(index, defaults) => {
                    if index.graph_index == self.root_graph_index {
                        Some(NodeEngineUpdate::NewDefaults(
                            defaults
                                .into_iter()
                                .filter_map(|(socket, value)| {
                                    if let Some(value) = value.as_value() {
                                        Some((index.node_index, socket, value))
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                        ))
                    } else {
                        None
                    }
                }
                ActionInvalidation::None => None,
                ActionInvalidation::NewNode(_) => None,
                ActionInvalidation::GraphModified(_) => None,
            })
            .collect()
    }

    pub fn commit(&mut self, actions: ActionBundle) -> Result<Vec<ActionInvalidation>, NodeError> {
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
            let is_new_bundle_property_related = new_actions.iter().all(Self::is_action_property_related);

            let is_current_bundle_property_related = self.history[self.place_in_history - 1]
                .actions
                .iter()
                .all(Self::is_action_property_related);

            if is_current_bundle_property_related && is_new_bundle_property_related {
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
            let (_, action_results) = to_rollback
                .actions
                .into_iter()
                .rev()
                .map(|action| self.rollback_action(action))
                .collect::<Result<Vec<(HistoryAction, Vec<ActionInvalidation>)>, NodeError>>()?
                .into_iter()
                .unzip::<HistoryAction, Vec<ActionInvalidation>, Vec<HistoryAction>, Vec<Vec<ActionInvalidation>>>();

            self.place_in_history -= 1;

            let invalidations = action_results.into_iter().flatten().collect();

            Ok(invalidations)
        } else {
            Ok(vec![])
        }
    }

    pub fn redo(&mut self) -> Result<Vec<ActionInvalidation>, NodeError> {
        if self.place_in_history < self.history.len() {
            let to_redo = self.history[self.place_in_history].clone();

            let (_, action_results) = to_redo
                .actions
                .into_iter()
                .rev()
                .map(|action| self.reapply_action(action))
                .collect::<Result<Vec<(HistoryAction, Vec<ActionInvalidation>)>, NodeError>>()?
                .into_iter()
                .unzip::<HistoryAction, Vec<ActionInvalidation>, Vec<HistoryAction>, Vec<Vec<ActionInvalidation>>>();

            self.place_in_history += 1;

            let invalidations = action_results.into_iter().flatten().collect();

            Ok(invalidations)
        } else {
            Ok(vec![])
        }
    }

    fn apply_action(&mut self, action: Action) -> Result<(HistoryAction, Vec<ActionInvalidation>), NodeError> {
        let mut warnings = vec![];

        let new_action = match action {
            Action::AddNode {
                graph: graph_index,
                node_type,
            } => {
                let (diff, invalidations) = self
                    .graph_manager
                    .create_node(&node_type, graph_index, &mut self.socket_registry)
                    .append_warnings(&mut warnings)?;

                (HistoryAction::GraphAction { diff }, invalidations)
            }
            Action::ConnectNodes { from, to, data } => {
                let (diff, invalidations) =
                    self.graph_manager
                        .connect_nodes(from, data.from_socket, to, data.to_socket)?;

                (HistoryAction::GraphAction { diff }, vec![invalidations])
            }
            Action::DisconnectNodes { from, to, data } => {
                let (diff, invalidations) =
                    self.graph_manager
                        .disconnect_nodes(from, data.from_socket, to, data.to_socket)?;

                (HistoryAction::GraphAction { diff }, vec![invalidations])
            }
            Action::RemoveNode { index } => {
                if index.graph_index == self.root_graph_index
                    && (index.node_index == self.io_nodes.input || index.node_index == self.io_nodes.output)
                {
                    return Err(NodeError::CannotDeleteRootNode);
                }

                let (diff, invalidations) = self.graph_manager.remove_node(index)?;

                (HistoryAction::GraphAction { diff }, vec![invalidations])
            }
            Action::ChangeNodeProperties { index, props } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let node = graph.get_node_mut(index.node_index)?;

                let before_props = node.set_properties(props.clone());
                let graph_diffs = graph.update_node_rows(index.node_index, &mut self.socket_registry)?;

                let graph_diff = GraphManagerDiff(
                    graph_diffs
                        .into_iter()
                        .map(|diff| DiffElement::ChildGraphDiff(index.graph_index, diff))
                        .collect(),
                );

                (
                    HistoryAction::ChangeNodeProperties {
                        index,
                        before: before_props,
                        after: props,
                        graph_diff,
                    },
                    vec![ActionInvalidation::GraphReindexNeeded(index.graph_index)],
                )
            }
            Action::ChangeNodeUiData { index, data } => self.reapply_action(HistoryAction::ChangeNodeUiData {
                index,
                before: HashMap::new(),
                after: data,
            })?,
            Action::ChangeNodeOverrides { index, overrides } => {
                self.reapply_action(HistoryAction::ChangeNodeOverrides {
                    index,
                    before: Vec::new(),
                    after: overrides,
                })?
            }
        };

        Ok(new_action)
    }

    fn reapply_action(&mut self, action: HistoryAction) -> Result<(HistoryAction, Vec<ActionInvalidation>), NodeError> {
        let mut action_result: Vec<ActionInvalidation> = vec![];

        let new_action = match action {
            HistoryAction::ChangeNodeProperties {
                index,
                before,
                after,
                graph_diff: _,
            } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let node = graph.get_node_mut(index.node_index)?;

                node.set_properties(after.clone());
                let graph_diffs = graph.update_node_rows(index.node_index, &mut self.socket_registry)?;

                let graph_diff = GraphManagerDiff(
                    graph_diffs
                        .into_iter()
                        .map(|diff| DiffElement::ChildGraphDiff(index.graph_index, diff))
                        .collect(),
                );

                action_result.push(ActionInvalidation::GraphReindexNeeded(index.graph_index));

                HistoryAction::ChangeNodeProperties {
                    index,
                    before,
                    after,
                    graph_diff,
                }
            }
            HistoryAction::ChangeNodeUiData {
                index,
                before: _,
                after,
            } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let node = graph.get_node_mut(index.node_index)?;

                let before = node.set_ui_data(after.clone());

                action_result.push(ActionInvalidation::GraphModified(index.graph_index));

                HistoryAction::ChangeNodeUiData { index, before, after }
            }
            HistoryAction::ChangeNodeOverrides { index, before, after } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let node = graph.get_node_mut(index.node_index)?;

                node.set_default_overrides(after.clone());

                let changed: Vec<(Socket, SocketValue)> = after
                    .iter()
                    .filter(|&after| !before.iter().any(|before| before == after))
                    .filter_map(NodeRow::to_socket_and_value)
                    .collect();

                action_result.push(ActionInvalidation::NewDefaults(index, changed));
                action_result.push(ActionInvalidation::GraphModified(index.graph_index));

                HistoryAction::ChangeNodeOverrides { index, before, after }
            }
            HistoryAction::GraphAction { diff } => {
                let cloned = diff.clone();
                action_result = self.graph_manager.reapply_action(diff)?;

                HistoryAction::GraphAction { diff: cloned }
            }
        };

        Ok((new_action, action_result))
    }

    fn rollback_action(
        &mut self,
        action: HistoryAction,
    ) -> Result<(HistoryAction, Vec<ActionInvalidation>), NodeError> {
        let mut action_result = vec![];

        let new_action = match action {
            HistoryAction::ChangeNodeProperties {
                index,
                before,
                after,
                graph_diff,
            } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let node = graph.get_node_mut(index.node_index)?;

                node.set_properties(before.clone());
                graph.update_node_rows(index.node_index, &mut self.socket_registry)?;

                let cloned = graph_diff.clone();
                action_result = self.graph_manager.rollback_action(graph_diff)?;
                action_result.push(ActionInvalidation::GraphReindexNeeded(index.graph_index));

                HistoryAction::ChangeNodeProperties {
                    index,
                    before,
                    after,
                    graph_diff: cloned,
                }
            }
            HistoryAction::ChangeNodeUiData { index, before, after } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let node = graph.get_node_mut(index.node_index)?;

                node.set_ui_data(before.clone());

                action_result.push(ActionInvalidation::GraphModified(index.graph_index));

                HistoryAction::ChangeNodeUiData { index, before, after }
            }
            HistoryAction::ChangeNodeOverrides { index, before, after } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let node = graph.get_node_mut(index.node_index)?;

                node.set_default_overrides(before.clone());

                let changed: Vec<(Socket, SocketValue)> = before
                    .iter()
                    .filter(|&before| !after.iter().any(|after| after == before))
                    .filter_map(NodeRow::to_socket_and_value)
                    .collect();

                action_result.push(ActionInvalidation::NewDefaults(index, changed));
                action_result.push(ActionInvalidation::GraphModified(index.graph_index));

                HistoryAction::ChangeNodeOverrides { index, before, after }
            }
            HistoryAction::GraphAction { diff } => {
                let cloned = diff.clone();
                action_result = self.graph_manager.rollback_action(diff)?;

                HistoryAction::GraphAction { diff: cloned }
            }
        };

        Ok((new_action, action_result))
    }
}

impl NodeState {
    pub fn to_json(&self) -> Value {
        json!({
            "graphManager": self.graph_manager,
            "rootGraphIndex": self.root_graph_index,
            "ioNodes": self.io_nodes,
            "socketRegistry": self.socket_registry
        })
    }

    pub fn load_state(
        &mut self,
        graph_manager: GraphManager,
        root_graph_index: GraphIndex,
        io_nodes: IoNodes,
        socket_registry: SocketRegistry,
    ) {
        self.history.clear();
        self.place_in_history = 0;
        self.graph_manager = graph_manager;
        self.root_graph_index = root_graph_index;
        self.io_nodes = io_nodes;
        self.socket_registry = socket_registry;
    }
}
