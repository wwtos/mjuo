use std::collections::HashMap;

use serde_json::{json, Value};

use crate::{
    engine::NodeEngine,
    errors::{NodeError, WarningBuilder, WarningProducer},
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex, GraphManager, GraphManagerDiff},
    node::{NodeIndex, NodeRow},
    node_graph::NodeConnection,
    property::Property,
    socket_registry::SocketRegistry,
    traversal::buffered_traverser::BufferedTraverser,
};
use rhai::Engine;

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

pub struct ActionInvalidations {
    pub graph_to_reindex: Option<GraphIndex>,
    pub graph_operated_on: Option<GraphIndex>,
    pub nodes_created: Vec<GlobalNodeIndex>,
    pub defaults_to_update: Option<Vec<GlobalNodeIndex>>,
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
    scripting_engine: Engine,
    root_graph_index: GraphIndex,
    output_node: NodeIndex,
    midi_in_node: NodeIndex,
    socket_registry: SocketRegistry,
}

impl NodeState {
    pub fn new(global_state: &GlobalState) -> Result<NodeState, NodeError> {
        let history = Vec::new();
        let place_in_history = 0;

        let graph_manager: GraphManager = GraphManager::new();
        let mut socket_registry = SocketRegistry::default();

        let root_graph_index = graph_manager.root_index();

        let (output_node, midi_in_node) = {
            let mut graph = graph_manager.get_graph(root_graph_index)?.graph.borrow_mut();

            let output_node = graph.add_node("OutputNode".into(), &mut socket_registry).unwrap().value;
            let midi_in_node = graph.add_node("MidiInNode".into(), &mut socket_registry).unwrap().value;

            (output_node.0, midi_in_node.0)
        };

        let scripting_engine: Engine = Engine::new_raw();
        let mut root_traverser = BufferedTraverser::new();

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
            scripting_engine,
            root_graph_index,
            output_node,
            midi_in_node,
            socket_registry,
        })
    }

    pub fn get_engine(&self, global_state: &GlobalState) -> Result<NodeEngine, NodeError> {
        let script_engine = rhai::Engine::new_raw();
        let resources = global_state.resources.read().unwrap();

        let traverser = BufferedTraverser::get_traverser(
            self.root_graph_index,
            &self.graph_manager,
            &script_engine,
            &resources,
            0,
            global_state.sound_config.clone(),
        )?;

        Ok(NodeEngine::new(
            traverser,
            script_engine,
            self.midi_in_node,
            self.output_node,
        ))
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

    pub fn notify_parents_of_graph_change(&mut self, graph_index: GraphIndex) -> Result<(), NodeError> {
        if graph_index != self.graph_manager.root_index() {
            let parent_nodes = self.graph_manager.get_graph_parents(graph_index)?;

            for GlobalNodeIndex {
                graph_index: parent_node_graph,
                node_index: parent_node_index,
            } in parent_nodes
            {
                let mut parent_node_graph = self.graph_manager.get_graph(parent_node_graph)?.graph.borrow_mut();
                // let subgraph = &mut self.graph_manager.get_graph(graph_index)?.graph.borrow_mut();

                // let node = parent_node_graph.get_node_mut(parent_node_index)?;
            }
        }

        Ok(())
    }
}

impl NodeState {
    fn handle_action_invalidations(
        &mut self,
        action_results: Vec<ActionInvalidations>,
        global_state: &GlobalState,
    ) -> Result<(Vec<GraphIndex>, Vec<GlobalNodeIndex>, Option<BufferedTraverser>), NodeError> {
        let mut graphs_to_reindex: Vec<GraphIndex> = Vec::new();
        let mut graphs_operated_on: Vec<GraphIndex> = Vec::new();
        let mut defaults_to_update: Vec<GlobalNodeIndex> = Vec::new();
        let mut nodes_created = Vec::new();

        let mut all_graphs_that_changed: Vec<GraphIndex> = Vec::new();

        for action_result in action_results {
            if let Some(graph_to_reindex) = action_result.graph_to_reindex {
                if !graphs_to_reindex.contains(&graph_to_reindex) {
                    graphs_to_reindex.push(graph_to_reindex)
                }

                if !all_graphs_that_changed.contains(&graph_to_reindex) {
                    all_graphs_that_changed.push(graph_to_reindex);
                }
            }

            if let Some(graph_operated_on) = action_result.graph_operated_on {
                if !graphs_operated_on.contains(&graph_operated_on) {
                    graphs_operated_on.push(graph_operated_on)
                }

                if !all_graphs_that_changed.contains(&graph_operated_on) {
                    all_graphs_that_changed.push(graph_operated_on);
                }
            }

            if let Some(new_defaults_to_update) = action_result.defaults_to_update {
                for new_default_to_update in new_defaults_to_update {
                    if !all_graphs_that_changed.contains(&new_default_to_update.graph_index) {
                        all_graphs_that_changed.push(new_default_to_update.graph_index);
                    }

                    if !defaults_to_update.contains(&new_default_to_update) {
                        defaults_to_update.push(new_default_to_update)
                    }
                }
            }

            nodes_created.extend(action_result.nodes_created);
        }

        let traverser = if !graphs_to_reindex.is_empty() || !defaults_to_update.is_empty() {
            Some(BufferedTraverser::get_traverser(
                self.root_graph_index,
                &self.graph_manager,
                &self.scripting_engine,
                &global_state.resources.read().unwrap(),
                0,
                global_state.sound_config.clone(),
            )?)
        } else {
            None
        };

        for graph_operated_on in graphs_operated_on {
            self.notify_parents_of_graph_change(graph_operated_on).unwrap();
        }

        Ok((all_graphs_that_changed, nodes_created, traverser))
    }

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

    pub fn commit(
        &mut self,
        actions: ActionBundle,
        global_state: &GlobalState,
    ) -> Result<(Vec<GraphIndex>, Vec<GlobalNodeIndex>, Option<BufferedTraverser>), NodeError> {
        let (mut new_actions, action_results) = actions
            .actions
            .into_iter()
            .map(|action| self.apply_action(action))
            .collect::<Result<Vec<(HistoryAction, ActionInvalidations)>, NodeError>>()?
            .into_iter()
            .unzip::<HistoryAction, ActionInvalidations, Vec<HistoryAction>, Vec<ActionInvalidations>>();

        if self.place_in_history < self.history.len() {
            self.history.truncate(self.place_in_history);
        }

        let (graphs_changed, nodes_changed, traverser) =
            self.handle_action_invalidations(action_results, global_state)?;

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

        Ok((graphs_changed, nodes_changed, traverser))
    }

    pub fn undo(
        &mut self,
        global_state: &GlobalState,
    ) -> Result<(Vec<GraphIndex>, Vec<GlobalNodeIndex>, Option<BufferedTraverser>), NodeError> {
        if self.place_in_history > 0 {
            let to_rollback = self.history[self.place_in_history - 1].clone();

            // roll back in reverse order
            let (_, action_results) = to_rollback
                .actions
                .into_iter()
                .rev()
                .map(|action| self.rollback_action(action))
                .collect::<Result<Vec<(HistoryAction, ActionInvalidations)>, NodeError>>()?
                .into_iter()
                .unzip::<HistoryAction, ActionInvalidations, Vec<HistoryAction>, Vec<ActionInvalidations>>();

            let graphs_changed = self.handle_action_invalidations(action_results, global_state)?;

            self.place_in_history -= 1;

            Ok(graphs_changed)
        } else {
            todo!("Make sure brand new traverser has input and output node");
            // Ok((Vec::new(), Vec::new(), Some(BufferedTraverser::new())))
        }
    }

    pub fn redo(
        &mut self,
        global_state: &GlobalState,
    ) -> Result<(Vec<GraphIndex>, Vec<GlobalNodeIndex>, Option<BufferedTraverser>), NodeError> {
        if self.place_in_history < self.history.len() {
            let to_redo = self.history[self.place_in_history].clone();

            let (_, action_results) = to_redo
                .actions
                .into_iter()
                .rev()
                .map(|action| self.reapply_action(action))
                .collect::<Result<Vec<(HistoryAction, ActionInvalidations)>, NodeError>>()?
                .into_iter()
                .unzip::<HistoryAction, ActionInvalidations, Vec<HistoryAction>, Vec<ActionInvalidations>>();

            let graphs_changed = self.handle_action_invalidations(action_results, global_state)?;

            self.place_in_history += 1;

            Ok(graphs_changed)
        } else {
            todo!("Make sure brand new traverser has input and output node");
            // Ok((Vec::new(), Vec::new(), Some(BufferedTraverser::new())))
        }
    }

    fn apply_action(&mut self, action: Action) -> Result<(HistoryAction, ActionInvalidations), NodeError> {
        println!("Applying action: {:?}", action);

        let mut warnings = WarningBuilder::new();

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

                (HistoryAction::GraphAction { diff }, invalidations)
            }
            Action::DisconnectNodes { from, to, data } => {
                let (diff, invalidations) =
                    self.graph_manager
                        .disconnect_nodes(from, data.from_socket, to, data.to_socket)?;

                (HistoryAction::GraphAction { diff }, invalidations)
            }
            Action::RemoveNode { index } => {
                let (diff, invalidations) = self.graph_manager.remove_node(index)?;

                (HistoryAction::GraphAction { diff }, invalidations)
            }
            Action::ChangeNodeProperties { index, props } => {
                self.reapply_action(HistoryAction::ChangeNodeProperties {
                    index,
                    before: HashMap::new(),
                    after: props,
                })?
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

    fn reapply_action(&mut self, action: HistoryAction) -> Result<(HistoryAction, ActionInvalidations), NodeError> {
        let mut action_result = ActionInvalidations {
            graph_to_reindex: None,
            graph_operated_on: None,
            defaults_to_update: None,
            nodes_created: vec![],
        };

        let mut warnings = WarningBuilder::new();

        let new_action = match action {
            HistoryAction::ChangeNodeProperties {
                index,
                before: _,
                after,
            } => {
                let before = {
                    let mut graph = self.graph_manager.get_graph(index.graph_index)?.graph.borrow_mut();
                    let node = graph.get_node_mut(index.node_index)?;

                    node.replace_properties(after.clone())
                };

                action_result.graph_operated_on = Some(index.graph_index);

                HistoryAction::ChangeNodeProperties { index, before, after }
            }
            HistoryAction::ChangeNodeUiData {
                index,
                before: _,
                after,
            } => {
                let mut graph = self.graph_manager.get_graph(index.graph_index)?.graph.borrow_mut();

                let node = graph.get_node_mut(index.node_index)?;

                let before = node.replace_ui_data(after.clone());

                action_result.graph_operated_on = Some(index.graph_index);

                if let Some(ref mut to_update) = action_result.defaults_to_update {
                    to_update.push(index.clone())
                } else {
                    action_result.defaults_to_update = Some(vec![index.clone()]);
                }

                HistoryAction::ChangeNodeUiData { index, before, after }
            }
            HistoryAction::ChangeNodeOverrides {
                index,
                before: _,
                after,
            } => {
                let mut graph = self.graph_manager.get_graph(index.graph_index)?.graph.borrow_mut();

                let node = graph.get_node_mut(index.node_index)?;

                let before = node.replace_default_overrides(after.clone());

                action_result.graph_operated_on = Some(index.graph_index);

                if let Some(ref mut to_update) = action_result.defaults_to_update {
                    to_update.push(index.clone())
                } else {
                    action_result.defaults_to_update = Some(vec![index.clone()]);
                }

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

    fn rollback_action(&mut self, action: HistoryAction) -> Result<(HistoryAction, ActionInvalidations), NodeError> {
        let mut action_result = ActionInvalidations {
            graph_to_reindex: None,
            graph_operated_on: None,
            defaults_to_update: None,
            nodes_created: vec![],
        };

        let new_action = match action {
            HistoryAction::ChangeNodeProperties { index, before, after } => {
                {
                    let mut graph = self.graph_manager.get_graph(index.graph_index)?.graph.borrow_mut();

                    let node = graph.get_node_mut(index.node_index)?;

                    node.set_properties(before.clone());
                }

                action_result.graph_operated_on = Some(index.graph_index);

                HistoryAction::ChangeNodeProperties { index, before, after }
            }
            HistoryAction::ChangeNodeUiData { index, before, after } => {
                let mut graph = self.graph_manager.get_graph(index.graph_index)?.graph.borrow_mut();

                let node = graph.get_node_mut(index.node_index)?;

                node.set_ui_data(before.clone());

                action_result.graph_operated_on = Some(index.graph_index);

                if let Some(ref mut to_update) = action_result.defaults_to_update {
                    to_update.push(index.clone())
                } else {
                    action_result.defaults_to_update = Some(vec![index.clone()]);
                }

                HistoryAction::ChangeNodeUiData { index, before, after }
            }
            HistoryAction::ChangeNodeOverrides { index, before, after } => {
                let mut graph = self.graph_manager.get_graph(index.graph_index)?.graph.borrow_mut();

                let node = graph.get_node_mut(index.node_index)?;

                node.set_default_overrides(before.clone());

                action_result.graph_operated_on = Some(index.graph_index);

                if let Some(ref mut to_update) = action_result.defaults_to_update {
                    to_update.push(index.clone())
                } else {
                    action_result.defaults_to_update = Some(vec![index.clone()]);
                }

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
            "graph_manager": self.graph_manager,
            "root_graph_index": self.root_graph_index,
            "output_node": self.output_node,
            "midi_in_node": self.midi_in_node
        })
    }

    pub fn load_state(
        &mut self,
        graph_manager: GraphManager,
        root_graph_index: GraphIndex,
        output_node: NodeIndex,
        midi_in_node: NodeIndex,
    ) {
        self.history.clear();
        self.place_in_history = 0;
        self.graph_manager = graph_manager;
        self.root_graph_index = root_graph_index;
        self.output_node = output_node;
        self.midi_in_node = midi_in_node;
    }
}
