use std::collections::HashMap;

use resource_manager::ResourceManager;
use serde_json::{json, Value};
use snafu::ResultExt;
use sound_engine::{sampling::sample::Sample, SoundConfig};

use crate::{
    connection::{MidiBundle, MidiSocketType, SocketType, StreamSocketType},
    errors::{JsonParserSnafu, NodeError, WarningBuilder, WarningProducer},
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex, GraphManager, GraphManagerDiff},
    node::{NodeIndex, NodeInitState, NodeRow},
    node_graph::NodeConnection,
    nodes::{midi_input::MidiInNode, output::OutputNode, variants::NodeVariant},
    property::Property,
    socket_registry::SocketRegistry,
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
    pub defaults_to_update: Option<Vec<GlobalNodeIndex>>,
}

#[derive(Clone, Debug)]
enum HistoryAction {
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

#[derive(Clone)]
struct HistoryActionBundle {
    actions: Vec<HistoryAction>,
}

#[derive(Clone)]
pub struct AssetBundle<'a> {
    pub samples: &'a ResourceManager<Sample>,
    // pub wavetables: &'a ResourceManager<Wavetable>,
}

pub struct NodeEngineState {
    history: Vec<HistoryActionBundle>,
    place_in_history: usize,
    graph_manager: GraphManager,
    sound_config: SoundConfig,
    socket_registry: SocketRegistry,
    scripting_engine: Engine,
    root_graph_index: GraphIndex,
    output_node: NodeIndex,
    midi_in_node: NodeIndex,
}

impl NodeEngineState {
    pub fn new(global_state: &GlobalState) -> Result<NodeEngineState, NodeError> {
        let history = Vec::new();
        let place_in_history = 0;
        let mut graph_manager = GraphManager::new();
        let mut socket_registry = SocketRegistry::new();
        let scripting_engine = Engine::new();

        SocketType::register_defaults(&mut socket_registry);

        let root_graph_index = graph_manager.root_index();

        let (output_node, midi_in_node) = {
            let mut graph = graph_manager.get_graph(root_graph_index)?.graph.borrow_mut();

            let output_node = graph
                .add_node(
                    NodeVariant::OutputNode(OutputNode::default()),
                    NodeInitState {
                        props: &HashMap::new(),
                        registry: &mut socket_registry,
                        script_engine: &scripting_engine,
                        global_state,
                    },
                )
                .unwrap()
                .value;
            let midi_in_node = graph
                .add_node(
                    NodeVariant::MidiInNode(MidiInNode::default()),
                    NodeInitState {
                        props: &HashMap::new(),
                        registry: &mut socket_registry,
                        script_engine: &scripting_engine,
                        global_state,
                    },
                )
                .unwrap()
                .value;

            (output_node.0, midi_in_node.0)
        };

        graph_manager.recalculate_traversal_for_graph(root_graph_index).unwrap();

        Ok(NodeEngineState {
            history,
            place_in_history,
            graph_manager,
            sound_config: global_state.sound_config.clone(),
            socket_registry,
            scripting_engine,
            root_graph_index,
            output_node,
            midi_in_node,
        })
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.place_in_history = 0;
    }

    pub fn get_graph_manager(&mut self) -> &mut GraphManager {
        &mut self.graph_manager
    }

    pub fn get_sound_config(&self) -> &SoundConfig {
        &self.sound_config
    }

    pub fn get_root_graph_index(&self) -> GraphIndex {
        self.root_graph_index
    }

    pub fn get_registry(&mut self) -> &mut SocketRegistry {
        &mut self.socket_registry
    }

    pub fn get_registry_and_engine(&mut self) -> (&mut SocketRegistry, &mut Engine) {
        (&mut self.socket_registry, &mut self.scripting_engine)
    }

    pub fn index_graph(&mut self, graph_index: GraphIndex) -> Result<(), NodeError> {
        self.graph_manager.recalculate_traversal_for_graph(graph_index)
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
                let subgraph = &mut self.graph_manager.get_graph(graph_index)?.graph.borrow_mut();

                let node = parent_node_graph.get_node_mut(parent_node_index)?;
                node.node_init_graph(subgraph);
            }
        }

        Ok(())
    }

    pub fn step(
        &mut self,
        current_time: i64,
        is_first_time: bool,
        midi_in: MidiBundle,
        global_state: &GlobalState,
    ) -> f32 {
        let root_graph = self.graph_manager.get_graph(self.graph_manager.root_index()).unwrap();

        let mut graph = root_graph.graph.borrow_mut();
        let traverser = &root_graph.traverser;

        let midi_in_node = graph.get_node_mut(self.midi_in_node).unwrap();
        midi_in_node.accept_midi_input(MidiSocketType::Default, midi_in);

        let traversal_errors = traverser.traverse(
            &mut graph,
            is_first_time,
            current_time,
            &self.scripting_engine,
            global_state,
        );

        if let Err(errors) = traversal_errors {
            println!("{:?}", errors);
        }

        let output_node = graph.get_node_mut(self.output_node).unwrap();

        output_node.get_stream_output(StreamSocketType::Audio)
    }
}

impl NodeEngineState {
    fn handle_action_invalidations(&mut self, action_results: Vec<ActionInvalidations>) -> Vec<GraphIndex> {
        let mut graphs_to_reindex: Vec<GraphIndex> = Vec::new();
        let mut graphs_operated_on: Vec<GraphIndex> = Vec::new();
        let mut defaults_to_update: Vec<GlobalNodeIndex> = Vec::new();

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
        }

        for to_reindex in graphs_to_reindex {
            self.index_graph(to_reindex).unwrap();
        }

        for graph_operated_on in graphs_operated_on {
            self.notify_parents_of_graph_change(graph_operated_on).unwrap();
        }

        for default_to_update in defaults_to_update {
            self.graph_manager
                .update_traversal_defaults(default_to_update.graph_index, vec![default_to_update.node_index]);
        }

        all_graphs_that_changed
    }

    fn is_action_property_related(action: &HistoryAction) -> bool {
        matches!(
            action,
            HistoryAction::ChangeNodeProperties { .. }
                | HistoryAction::ChangeNodeUiData { .. }
                | HistoryAction::ChangeNodeOverrides { .. }
        )
    }

    fn get_history_ref(&self) -> &Vec<HistoryActionBundle> {
        &self.history
    }

    pub fn commit(&mut self, actions: ActionBundle, global_state: &GlobalState) -> Result<Vec<GraphIndex>, NodeError> {
        let (mut new_actions, action_results) = actions
            .actions
            .into_iter()
            .map(|action| self.apply_action(action, global_state))
            .collect::<Result<Vec<(HistoryAction, ActionInvalidations)>, NodeError>>()?
            .into_iter()
            .unzip::<HistoryAction, ActionInvalidations, Vec<HistoryAction>, Vec<ActionInvalidations>>();

        if self.place_in_history < self.history.len() {
            self.history.truncate(self.place_in_history);
        }

        let graphs_changed = self.handle_action_invalidations(action_results);

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

        Ok(graphs_changed)
    }

    pub fn undo(&mut self, global_state: &GlobalState) -> Result<Vec<GraphIndex>, NodeError> {
        if self.place_in_history > 0 {
            let to_rollback = self.history[self.place_in_history - 1].clone();

            // roll back in reverse order
            let (_, action_results) = to_rollback
                .actions
                .into_iter()
                .rev()
                .map(|action| self.rollback_action(action, global_state))
                .collect::<Result<Vec<(HistoryAction, ActionInvalidations)>, NodeError>>()?
                .into_iter()
                .unzip::<HistoryAction, ActionInvalidations, Vec<HistoryAction>, Vec<ActionInvalidations>>();

            let graphs_changed = self.handle_action_invalidations(action_results);

            self.place_in_history -= 1;

            Ok(graphs_changed)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn redo(&mut self, global_state: &GlobalState) -> Result<Vec<GraphIndex>, NodeError> {
        if self.place_in_history < self.history.len() {
            let to_redo = self.history[self.place_in_history].clone();

            let (_, action_results) = to_redo
                .actions
                .into_iter()
                .rev()
                .map(|action| self.reapply_action(action, global_state))
                .collect::<Result<Vec<(HistoryAction, ActionInvalidations)>, NodeError>>()?
                .into_iter()
                .unzip::<HistoryAction, ActionInvalidations, Vec<HistoryAction>, Vec<ActionInvalidations>>();

            let graphs_changed = self.handle_action_invalidations(action_results);

            self.place_in_history += 1;

            Ok(graphs_changed)
        } else {
            Ok(Vec::new())
        }
    }

    fn apply_action(
        &mut self,
        action: Action,
        global_state: &GlobalState,
    ) -> Result<(HistoryAction, ActionInvalidations), NodeError> {
        println!("Applying action: {:?}", action);

        let mut warnings = WarningBuilder::new();

        let new_action = match action {
            Action::AddNode {
                graph: graph_index,
                node_type,
            } => {
                let (diff, invalidations) = self
                    .graph_manager
                    .create_node(
                        &node_type,
                        graph_index,
                        &self.sound_config,
                        NodeInitState {
                            props: &HashMap::new(),
                            registry: &mut self.socket_registry,
                            script_engine: &self.scripting_engine,
                            global_state,
                        },
                    )
                    .append_warnings(&mut warnings)?;

                (HistoryAction::GraphAction { diff }, invalidations)
            }
            Action::ConnectNodes { from, to, data } => {
                let (diff, invalidations) =
                    self.graph_manager
                        .connect_nodes(from, data.from_socket_type, to, data.to_socket_type)?;

                (HistoryAction::GraphAction { diff }, invalidations)
            }
            Action::DisconnectNodes { from, to, data } => {
                let (diff, invalidations) =
                    self.graph_manager
                        .disconnect_nodes(from, data.from_socket_type, to, data.to_socket_type)?;

                (HistoryAction::GraphAction { diff }, invalidations)
            }
            Action::RemoveNode { index } => {
                let (diff, invalidations) = self.graph_manager.remove_node(index)?;

                (HistoryAction::GraphAction { diff }, invalidations)
            }
            Action::ChangeNodeProperties { index, props } => self.reapply_action(
                HistoryAction::ChangeNodeProperties {
                    index,
                    before: HashMap::new(),
                    after: props,
                },
                global_state,
            )?,
            Action::ChangeNodeUiData { index, data } => self.reapply_action(
                HistoryAction::ChangeNodeUiData {
                    index,
                    before: HashMap::new(),
                    after: data,
                },
                global_state,
            )?,
            Action::ChangeNodeOverrides { index, overrides } => self.reapply_action(
                HistoryAction::ChangeNodeOverrides {
                    index,
                    before: Vec::new(),
                    after: overrides,
                },
                global_state,
            )?,
        };

        Ok(new_action)
    }

    fn reapply_action(
        &mut self,
        action: HistoryAction,
        global_state: &GlobalState,
    ) -> Result<(HistoryAction, ActionInvalidations), NodeError> {
        println!("Reapplying action: {:?}", action);

        let mut action_result = ActionInvalidations {
            graph_to_reindex: None,
            graph_operated_on: None,
            defaults_to_update: None,
        };

        let mut warnings = WarningBuilder::new();

        let new_action = match action {
            HistoryAction::ChangeNodeProperties {
                index,
                before: _,
                after,
            } => {
                let mut graph = self.graph_manager.get_graph(index.graph_index)?.graph.borrow_mut();
                let node = graph.get_node_mut(index.node_index)?;

                let before = node.replace_properties(after.clone());

                graph.init_node(
                    index.node_index,
                    NodeInitState {
                        props: &HashMap::new(),
                        registry: &mut self.socket_registry,
                        script_engine: &self.scripting_engine,
                        global_state,
                    },
                    false,
                )?;

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
                self.graph_manager.reapply_action(diff)?;

                HistoryAction::GraphAction { diff: cloned }
            }
        };

        Ok((new_action, action_result))
    }

    fn rollback_action(
        &mut self,
        action: HistoryAction,
        global_state: &GlobalState,
    ) -> Result<(HistoryAction, ActionInvalidations), NodeError> {
        println!("Rolling back action: {:?}", action);

        let mut action_result = ActionInvalidations {
            graph_to_reindex: None,
            graph_operated_on: None,
            defaults_to_update: None,
        };

        let new_action = match action {
            HistoryAction::ChangeNodeProperties { index, before, after } => {
                let mut graph = self.graph_manager.get_graph(index.graph_index)?.graph.borrow_mut();

                let node = graph.get_node_mut(index.node_index)?;

                node.set_properties(before.clone());

                graph.init_node(
                    index.node_index,
                    NodeInitState {
                        props: &HashMap::new(),
                        registry: &mut self.socket_registry,
                        script_engine: &self.scripting_engine,
                        global_state,
                    },
                    false,
                )?;

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
                self.graph_manager.rollback_action(diff)?;

                HistoryAction::GraphAction { diff: cloned }
            }
        };

        Ok((new_action, action_result))
    }
}

impl NodeEngineState {
    pub fn to_json(&self) -> Result<Value, NodeError> {
        Ok(json!({
            "graph_manager": self.graph_manager,
            "socket_registry": self.socket_registry,
            "root_graph_index": self.root_graph_index,
            "output_node": self.output_node,
            "midi_in_node": self.midi_in_node
        }))
    }

    pub fn apply_json(&mut self, mut json: Value, global_state: &GlobalState) -> Result<(), NodeError> {
        self.history.clear();
        self.place_in_history = 0;
        self.graph_manager = serde_json::from_value(json["graph_manager"].take()).context(JsonParserSnafu)?;
        self.socket_registry = serde_json::from_value(json["socket_registry"].take()).context(JsonParserSnafu)?;
        self.root_graph_index = serde_json::from_value(json["root_graph_index"].take()).context(JsonParserSnafu)?;
        self.output_node = serde_json::from_value(json["output_node"].take()).context(JsonParserSnafu)?;
        self.midi_in_node = serde_json::from_value(json["midi_in_node"].take()).context(JsonParserSnafu)?;

        let NodeEngineState {
            graph_manager,
            ref mut socket_registry,
            scripting_engine,
            sound_config,
            ..
        } = self;

        graph_manager.post_deserialization(
            NodeInitState {
                props: &HashMap::new(),
                registry: socket_registry,
                script_engine: scripting_engine,
                global_state,
            },
            sound_config,
        )?;

        Ok(())
    }
}
