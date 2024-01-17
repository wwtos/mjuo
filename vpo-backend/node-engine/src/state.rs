use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
    time::Duration,
};

use common::SeaHashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sound_engine::SoundConfig;

use crate::{
    connection::{Socket, SocketValue},
    errors::{NodeError, WarningExt},
    graph_manager::{GlobalNodeIndex, GraphIndex, GraphManager, GraphManagerDiff},
    io_routing::IoRoutes,
    node::buffered_traverser::BufferedTraverser,
    node::{NodeGetIoContext, NodeIndex, NodeRow, NodeState},
    node_graph::{NodeConnectionData, NodeGraph},
    nodes::variant_io,
    property::Property,
    resources::Resources,
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
        ui_data: SeaHashMap<String, Value>,
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
        props: SeaHashMap<String, Property>,
    },
    ChangeNodeUiData {
        index: GlobalNodeIndex,
        #[serde(rename = "uiData")]
        ui_data: SeaHashMap<String, Value>,
    },
    ChangeNodeOverrides {
        index: GlobalNodeIndex,
        overrides: Vec<NodeRow>,
    },
    ChangeRouteRules {
        #[serde(rename = "newRules")]
        new_rules: IoRoutes,
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

#[derive(Debug, PartialEq)]
pub enum ActionInvalidation {
    GraphReindexNeeded(GraphIndex),
    GraphModified(GraphIndex),
    NewDefaults(GlobalNodeIndex, Vec<(Socket, SocketValue)>),
    NewRouteRules { last_rules: IoRoutes, new_rules: IoRoutes },
    NewNode(GlobalNodeIndex),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionCategory {
    Separate,
    Mergable,
}

#[derive(Debug, Clone)]
pub enum FromNodeEngine {
    NodeStateUpdates(Vec<(NodeIndex, NodeState)>),
    RequestedStateUpdates(Vec<(NodeIndex, serde_json::Value)>),
    GraphStateRequested,
}

#[derive(Clone, Debug)]
pub enum HistoryAction {
    GraphAction {
        diff: GraphManagerDiff,
        category: ActionCategory,
    },
    RoutesAction {
        old: IoRoutes,
        new: IoRoutes,
    },
}

impl HistoryAction {
    pub fn category(&self) -> ActionCategory {
        match self {
            HistoryAction::GraphAction { category, .. } => category.clone(),
            HistoryAction::RoutesAction { .. } => ActionCategory::Separate,
        }
    }
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
    io_routing: IoRoutes,
    sound_config: SoundConfig,
    default_channel_count: usize,
}

impl GraphState {
    pub fn new(sound_config: SoundConfig) -> GraphState {
        let default_channel_count = 2;

        let history = Vec::new();
        let place_in_history = 0;

        let graph_manager: GraphManager = GraphManager::new(default_channel_count);
        let root_graph_index = graph_manager.root_index();

        GraphState {
            history,
            place_in_history,
            graph_manager,
            root_graph_index,
            io_routing: IoRoutes::default(),
            default_channel_count,
            sound_config,
        }
    }

    pub fn get_traverser(&self, resources: &Resources) -> Result<BufferedTraverser, NodeError> {
        let traverser = BufferedTraverser::new(
            self.sound_config.clone(),
            &self.graph_manager,
            self.root_graph_index,
            &resources,
            Duration::ZERO,
        )?;

        Ok(traverser)
    }

    pub fn create_traverser(&self, resources: &Resources) -> Result<BufferedTraverser, NodeError> {
        BufferedTraverser::new(
            self.sound_config.clone(),
            &self.graph_manager,
            self.root_graph_index,
            &resources,
            Duration::ZERO,
        )
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

    pub fn get_sound_config(&self) -> SoundConfig {
        self.sound_config.clone()
    }

    pub fn get_route_rules(&self) -> IoRoutes {
        self.io_routing.clone()
    }

    pub fn set_route_rules(&mut self, routes: IoRoutes) {
        self.io_routing = routes;
    }
}

impl GraphState {
    pub fn get_history(&self) -> &Vec<HistoryActionBundle> {
        &self.history
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
            let is_new_bundle_mergable = new_actions.iter().all(|x| x.category() == ActionCategory::Mergable);
            let is_current_bundle_mergable = self.history[self.place_in_history - 1]
                .actions
                .iter()
                .all(|x| x.category() == ActionCategory::Mergable);

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
                    HistoryAction::GraphAction {
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
                    HistoryAction::GraphAction {
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
                    HistoryAction::GraphAction {
                        diff,
                        category: ActionCategory::Separate,
                    },
                    vec![invalidations],
                )
            }
            Action::RemoveNode { index } => {
                let (diff, invalidation) = self.graph_manager.remove_node(index)?;

                (
                    HistoryAction::GraphAction {
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
                    HistoryAction::GraphAction {
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
                    HistoryAction::GraphAction {
                        diff: GraphManagerDiff::from_graph_diffs(index.graph_index, diffs),
                        category: ActionCategory::Mergable,
                    },
                    vec![],
                )
            }
            Action::ChangeNodeOverrides { index, overrides } => {
                let graph = self.graph_manager.get_graph_mut(index.graph_index)?;
                let mut modified_node = graph.get_node(index.node_index)?.clone();

                modified_node.set_default_overrides(overrides.clone());

                let diffs = graph.update_node(index.node_index, modified_node)?;

                let new_defaults: Vec<_> = overrides
                    .iter()
                    .map(|row| {
                        let (socket, val) = row.to_socket_and_value().unwrap();

                        (socket.clone(), val)
                    })
                    .collect();

                (
                    HistoryAction::GraphAction {
                        diff: GraphManagerDiff::from_graph_diffs(index.graph_index, diffs),
                        category: ActionCategory::Mergable,
                    },
                    vec![ActionInvalidation::NewDefaults(index, new_defaults)],
                )
            }
            Action::ChangeRouteRules { new_rules } => {
                // ensure rules are all unique
                let mut new_rules_set = BTreeSet::new();

                for rule in new_rules.devices.iter() {
                    let unique = new_rules_set.insert((&rule.name, rule.device_type, rule.device_direction));

                    if !unique {
                        return Err(NodeError::RouteRulesNotUnique { rules: new_rules });
                    }
                }

                let old_rules = self.get_route_rules();

                self.io_routing = new_rules.clone();

                (
                    HistoryAction::RoutesAction {
                        old: old_rules.clone(),
                        new: new_rules.clone(),
                    },
                    vec![ActionInvalidation::NewRouteRules {
                        last_rules: old_rules,
                        new_rules,
                    }],
                )
            }
        };

        Ok(new_action)
    }

    fn reapply_action(&mut self, action: HistoryAction) -> Result<Vec<ActionInvalidation>, NodeError> {
        match action {
            HistoryAction::GraphAction { diff, .. } => {
                let invalidations = self.graph_manager.reapply_action(diff)?;

                Ok(invalidations)
            }
            HistoryAction::RoutesAction { new, .. } => {
                let last_rules = mem::replace(&mut self.io_routing, new.clone());

                Ok(vec![ActionInvalidation::NewRouteRules {
                    last_rules,
                    new_rules: new.clone(),
                }])
            }
        }
    }

    fn rollback_action(&mut self, action: HistoryAction) -> Result<Vec<ActionInvalidation>, NodeError> {
        match action {
            HistoryAction::GraphAction { diff, .. } => {
                let invalidations = self.graph_manager.rollback_action(diff)?;

                Ok(invalidations)
            }
            HistoryAction::RoutesAction { old, .. } => {
                let last_rules = mem::replace(&mut self.io_routing, old.clone());

                Ok(vec![ActionInvalidation::NewRouteRules {
                    last_rules,
                    new_rules: old.clone(),
                }])
            }
        }
    }
}

impl GraphState {
    pub fn to_json(&self) -> Value {
        json!({
            "graphManager": self.graph_manager,
            "rootGraphIndex": self.root_graph_index,
            "defaultChannelCount": self.default_channel_count,
            "ioRouting": self.io_routing
        })
    }

    pub fn load_state(&mut self, graph_manager: GraphManager, root_graph_index: GraphIndex, routing: IoRoutes) {
        self.history.clear();
        self.place_in_history = 0;
        self.graph_manager = graph_manager;
        self.root_graph_index = root_graph_index;
        self.io_routing = routing;

        self.graph_manager.set_default_channel_count(self.default_channel_count);

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
