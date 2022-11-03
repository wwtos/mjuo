use std::collections::HashMap;

use asset_manager::AssetManager;
use serde_json::{json, Value};
use snafu::ResultExt;
use sound_engine::{midi::messages::MidiData, MonoSample, SoundConfig};

use crate::{
    connection::{Connection, MidiSocketType, SocketType, StreamSocketType},
    errors::{JsonParserSnafu, NodeError, WarningBuilder},
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex, GraphManager, NodeGraphWrapper},
    node::{NodeIndex, NodeInitState, NodeRow},
    nodes::{midi_input::MidiInNode, output::OutputNode, variants::NodeVariant},
    property::Property,
    socket_registry::SocketRegistry,
};
use rhai::Engine;

#[derive(Clone, Debug)]
pub enum Action {
    CreateNode {
        node_type: String,
        graph_index: GraphIndex,
        node_index: Option<NodeIndex>,
        child_graph_index: Option<GraphIndex>,
        child_graph_io_indexes: Option<(NodeIndex, NodeIndex)>,
    },
    RemoveNode {
        node_type: Option<String>,
        index: GlobalNodeIndex,
        child_graph_index: Option<GraphIndex>,
        child_graph_io_indexes: Option<(NodeIndex, NodeIndex)>,
        connections: Option<Vec<Connection>>,
        serialized: Option<Value>,
    },
    ChangeNodeProperties {
        index: GlobalNodeIndex,
        before: Option<HashMap<String, Property>>,
        after: HashMap<String, Property>,
    },
    ChangeNodeUiData {
        index: GlobalNodeIndex,
        before: Option<HashMap<String, Value>>,
        after: HashMap<String, Value>,
    },
    ChangeNodeOverrides {
        index: GlobalNodeIndex,
        before: Option<Vec<NodeRow>>,
        after: Vec<NodeRow>,
    },
    AddConnection {
        graph_index: GraphIndex,
        connection: Connection,
    },
    RemoveConnection {
        graph_index: GraphIndex,
        connection: Connection,
    },
}

pub struct ActionResult {
    pub graph_to_reindex: Option<GraphIndex>,
    pub graph_operated_on: Option<GraphIndex>,
    pub defaults_to_update: Option<Vec<GlobalNodeIndex>>,
}

#[derive(Clone)]
pub struct ActionBundle {
    actions: Vec<Action>,
}

impl ActionBundle {
    pub fn new(actions: Vec<Action>) -> ActionBundle {
        ActionBundle { actions: actions }
    }
}

#[derive(Clone)]
pub struct AssetBundle<'a> {
    pub samples: &'a AssetManager<MonoSample>,
}

pub struct NodeEngineState {
    history: Vec<ActionBundle>,
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
    pub fn new(sound_config: SoundConfig, global_state: &GlobalState) -> NodeEngineState {
        let history = Vec::new();
        let place_in_history = 0;
        let mut graph_manager = GraphManager::new();
        let mut socket_registry = SocketRegistry::new();
        let scripting_engine = Engine::new();

        SocketType::register_defaults(&mut socket_registry);

        let root_graph_index = graph_manager.new_graph();

        let (output_node, midi_in_node) = {
            let graph = &mut graph_manager.get_graph_wrapper_mut(root_graph_index).unwrap().graph;

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

            (output_node, midi_in_node)
        };

        graph_manager
            .recalculate_traversal_for_graph(&root_graph_index)
            .unwrap();

        NodeEngineState {
            history,
            place_in_history,
            graph_manager,
            sound_config,
            socket_registry,
            scripting_engine,
            root_graph_index,
            output_node,
            midi_in_node,
        }
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

    pub fn get_registry(&mut self) -> &mut SocketRegistry {
        &mut self.socket_registry
    }

    pub fn get_registry_and_engine(&mut self) -> (&mut SocketRegistry, &mut Engine) {
        (&mut self.socket_registry, &mut self.scripting_engine)
    }

    pub fn index_graph(&mut self, graph_index: &GraphIndex) -> Result<(), NodeError> {
        self.graph_manager.recalculate_traversal_for_graph(graph_index)
    }

    pub fn notify_parents_of_graph_change(&mut self, graph_index: &GraphIndex) -> Result<(), NodeError> {
        if graph_index != &0 {
            let parent_nodes = self.graph_manager.get_subgraph_parent_nodes(*graph_index);

            for GlobalNodeIndex {
                graph_index: parent_node_graph,
                node_index: parent_node_index,
            } in parent_nodes
            {
                let parent_node_graph = &mut self
                    .graph_manager
                    .get_graph_wrapper_mut(parent_node_graph)
                    .ok_or(NodeError::GraphDoesNotExist {
                        graph_index: *graph_index,
                    })?
                    .graph;
                let subgraph = &mut self
                    .graph_manager
                    .get_graph_wrapper_mut(*graph_index)
                    .ok_or(NodeError::GraphDoesNotExist {
                        graph_index: *graph_index,
                    })?
                    .graph;

                let node = parent_node_graph
                    .get_node_mut(&parent_node_index)
                    .ok_or(NodeError::NodeDoesNotExist {
                        node_index: parent_node_index,
                    })?;
                node.node_init_graph(subgraph);
            }
        }

        Ok(())
    }

    pub fn step(
        &mut self,
        current_time: i64,
        is_first_time: bool,
        midi_in: Vec<MidiData>,
        samples: &AssetManager<MonoSample>,
    ) -> f32 {
        let NodeGraphWrapper {
            ref mut graph,
            ref traverser,
            ..
        } = &mut *self.graph_manager.get_graph_wrapper_mut(self.root_graph_index).unwrap();

        let midi_in_node = graph.get_node_mut(&self.midi_in_node).unwrap();
        midi_in_node.accept_midi_input(&MidiSocketType::Default, midi_in);

        let traversal_errors = traverser.traverse(graph, is_first_time, current_time, &self.scripting_engine, samples);

        if let Err(errors) = traversal_errors {
            println!("{:?}", errors);
        }

        let output_node = graph.get_node_mut(&self.output_node).unwrap();
        let audio = output_node.get_stream_output(&StreamSocketType::Audio);

        audio
    }
}

impl NodeEngineState {
    fn handle_action_results(&mut self, action_results: Vec<ActionResult>) -> Vec<GraphIndex> {
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
            self.index_graph(&to_reindex).unwrap();
        }

        for graph_operated_on in graphs_operated_on {
            self.notify_parents_of_graph_change(&graph_operated_on).unwrap();
        }

        for default_to_update in defaults_to_update {
            self.graph_manager
                .update_traversal_defaults(default_to_update.graph_index, vec![default_to_update.node_index]);
        }

        all_graphs_that_changed
    }

    pub fn is_action_property_related(action: &Action) -> bool {
        match action {
            Action::ChangeNodeProperties { .. } => true,
            Action::ChangeNodeUiData { .. } => true,
            Action::ChangeNodeOverrides { .. } => true,
            _ => false,
        }
    }

    pub fn commit(&mut self, actions: ActionBundle, global_state: &GlobalState) -> Result<Vec<GraphIndex>, NodeError> {
        let is_new_bundle_property_related = actions.actions.iter().all(Self::is_action_property_related);

        let (mut new_actions, action_results) = actions
            .actions
            .into_iter()
            .map(|action| self.apply_action(action, global_state))
            .collect::<Result<Vec<(Action, ActionResult)>, NodeError>>()?
            .into_iter()
            .unzip::<Action, ActionResult, Vec<Action>, Vec<ActionResult>>();

        if self.place_in_history < self.history.len() {
            self.history.truncate(self.place_in_history);
        }

        let graphs_changed = self.handle_action_results(action_results);

        // determine whether to add a new action bundle, or to concatinate it to the current
        // action bundle
        if !self.history.is_empty() {
            let is_current_bundle_property_related = self.history[self.place_in_history - 1]
                .actions
                .iter()
                .all(Self::is_action_property_related);

            if is_current_bundle_property_related && is_new_bundle_property_related {
                self.history[self.place_in_history - 1].actions.append(&mut new_actions);
            } else {
                self.history.push(ActionBundle { actions: new_actions });

                self.place_in_history += 1;
            }
        } else {
            self.history.push(ActionBundle { actions: new_actions });

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
                .collect::<Result<Vec<(Action, ActionResult)>, NodeError>>()?
                .into_iter()
                .unzip::<Action, ActionResult, Vec<Action>, Vec<ActionResult>>();

            let graphs_changed = self.handle_action_results(action_results);

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
                .map(|action| self.apply_action(action, global_state))
                .collect::<Result<Vec<(Action, ActionResult)>, NodeError>>()?
                .into_iter()
                .unzip::<Action, ActionResult, Vec<Action>, Vec<ActionResult>>();

            let graphs_changed = self.handle_action_results(action_results);

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
    ) -> Result<(Action, ActionResult), NodeError> {
        println!("Applying action: {:?}", action);

        let mut action_result = ActionResult {
            graph_to_reindex: None,
            graph_operated_on: None,
            defaults_to_update: None,
        };

        let mut warnings = WarningBuilder::new();

        let new_action: Action = match action {
            Action::CreateNode {
                node_type,
                graph_index,
                node_index,
                child_graph_index,
                child_graph_io_indexes,
            } => {
                action_result.graph_operated_on = Some(graph_index);

                let result = self.graph_manager.create_node_at_index(
                    &node_type,
                    graph_index,
                    node_index,
                    child_graph_index,
                    child_graph_io_indexes,
                    &self.sound_config,
                    NodeInitState {
                        props: &HashMap::new(),
                        registry: &mut self.socket_registry,
                        script_engine: &self.scripting_engine,
                        global_state,
                    },
                )?;

                warnings.append_warnings(result.warnings);

                result.value
            }
            Action::RemoveNode { index, .. } => {
                let result = self.graph_manager.remove_node(&index)?;

                action_result.graph_operated_on = Some(index.graph_index);
                action_result.graph_to_reindex = Some(index.graph_index);

                result
            }
            Action::ChangeNodeProperties {
                index,
                before: _,
                after,
            } => {
                let mut graph = self.graph_manager.get_graph_wrapper_mut(index.graph_index).ok_or(
                    NodeError::GraphDoesNotExist {
                        graph_index: index.graph_index,
                    },
                )?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist {
                    node_index: index.node_index,
                })?;

                let before = node.replace_properties(after.clone());

                graph.graph.init_node(
                    &index.node_index,
                    NodeInitState {
                        props: &HashMap::new(),
                        registry: &mut self.socket_registry,
                        script_engine: &self.scripting_engine,
                        global_state,
                    },
                    false,
                )?;

                action_result.graph_operated_on = Some(index.graph_index);

                Action::ChangeNodeProperties {
                    index: index,
                    before: Some(before),
                    after: after,
                }
            }
            Action::ChangeNodeUiData {
                index,
                before: _,
                after,
            } => {
                let mut graph = self.graph_manager.get_graph_wrapper_mut(index.graph_index).ok_or(
                    NodeError::GraphDoesNotExist {
                        graph_index: index.graph_index,
                    },
                )?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist {
                    node_index: index.node_index,
                })?;

                let before = node.replace_ui_data(after.clone());

                action_result.graph_operated_on = Some(index.graph_index);

                if let Some(ref mut to_update) = action_result.defaults_to_update {
                    to_update.push(index.clone())
                } else {
                    action_result.defaults_to_update = Some(vec![index.clone()]);
                }

                Action::ChangeNodeUiData {
                    index: index,
                    before: Some(before),
                    after: after,
                }
            }
            Action::ChangeNodeOverrides {
                index,
                before: _,
                after,
            } => {
                let mut graph = self.graph_manager.get_graph_wrapper_mut(index.graph_index).ok_or(
                    NodeError::GraphDoesNotExist {
                        graph_index: index.graph_index,
                    },
                )?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist {
                    node_index: index.node_index,
                })?;

                let before = node.replace_default_overrides(after.clone());

                action_result.graph_operated_on = Some(index.graph_index);

                if let Some(ref mut to_update) = action_result.defaults_to_update {
                    to_update.push(index.clone())
                } else {
                    action_result.defaults_to_update = Some(vec![index.clone()]);
                }

                Action::ChangeNodeOverrides {
                    index: index,
                    before: Some(before),
                    after: after,
                }
            }
            Action::AddConnection {
                graph_index,
                connection,
            } => {
                let graph = &mut self
                    .graph_manager
                    .get_graph_wrapper_mut(graph_index)
                    .ok_or(NodeError::GraphDoesNotExist {
                        graph_index: graph_index,
                    })?
                    .graph;

                graph.connect(
                    &connection.from_node,
                    &connection.from_socket_type,
                    &connection.to_node,
                    &connection.to_socket_type,
                )?;

                action_result.graph_operated_on = Some(graph_index);
                action_result.graph_to_reindex = Some(graph_index);

                Action::AddConnection {
                    graph_index,
                    connection,
                }
            }
            Action::RemoveConnection {
                graph_index,
                connection,
            } => {
                let graph = &mut self
                    .graph_manager
                    .get_graph_wrapper_mut(graph_index)
                    .ok_or(NodeError::GraphDoesNotExist {
                        graph_index: graph_index,
                    })?
                    .graph;

                graph.disconnect(
                    &connection.from_node,
                    &connection.from_socket_type,
                    &connection.to_node,
                    &connection.to_socket_type,
                )?;

                action_result.graph_operated_on = Some(graph_index);
                action_result.graph_to_reindex = Some(graph_index);

                Action::RemoveConnection {
                    graph_index,
                    connection,
                }
            }
        };

        Ok((new_action, action_result))
    }

    fn rollback_action(
        &mut self,
        action: Action,
        global_state: &GlobalState,
    ) -> Result<(Action, ActionResult), NodeError> {
        println!("Rolling back action: {:?}", action);

        let mut action_result = ActionResult {
            graph_to_reindex: None,
            graph_operated_on: None,
            defaults_to_update: None,
        };

        let new_action = match action {
            Action::CreateNode {
                node_type,
                graph_index,
                node_index,
                child_graph_index,
                child_graph_io_indexes,
            } => {
                let node_index = node_index.ok_or(NodeError::ActionRollbackFieldMissing {
                    missing_field: "node_index".to_string(),
                })?;

                action_result.graph_operated_on = Some(graph_index);
                action_result.graph_to_reindex = Some(graph_index);

                self.graph_manager
                    .remove_node(&GlobalNodeIndex {
                        graph_index: graph_index,
                        node_index: node_index.clone(),
                    })
                    .map(|_| Action::CreateNode {
                        node_type: node_type.clone(),
                        graph_index: graph_index.clone(),
                        node_index: Some(node_index),
                        child_graph_index: child_graph_index.clone(),
                        child_graph_io_indexes: child_graph_io_indexes.clone(),
                    })
            }
            Action::RemoveNode {
                node_type,
                index,
                child_graph_index,
                child_graph_io_indexes,
                connections,
                serialized,
            } => {
                // unwrap all the Option fields (should be present for a rollback)
                let node_type = node_type.ok_or(NodeError::ActionRollbackFieldMissing {
                    missing_field: "node_type".to_string(),
                })?;
                let connections = connections.ok_or(NodeError::ActionRollbackFieldMissing {
                    missing_field: "connections".to_string(),
                })?;
                let serialized = serialized.ok_or(NodeError::ActionRollbackFieldMissing {
                    missing_field: "serialized".to_string(),
                })?;

                action_result.graph_operated_on = Some(index.graph_index);

                self.graph_manager.create_node_at_index(
                    &node_type,
                    index.graph_index,
                    Some(index.node_index),
                    child_graph_index,
                    child_graph_io_indexes,
                    &self.sound_config,
                    NodeInitState {
                        props: &HashMap::new(),
                        registry: &mut self.socket_registry,
                        script_engine: &self.scripting_engine,
                        global_state,
                    },
                )?;

                // connect everything back up
                let mut graph = self.graph_manager.get_graph_wrapper_mut(index.graph_index).ok_or(
                    NodeError::GraphDoesNotExist {
                        graph_index: index.graph_index,
                    },
                )?;

                for connection in connections.iter() {
                    graph.graph.connect(
                        &connection.from_node,
                        &connection.from_socket_type,
                        &connection.to_node,
                        &connection.to_socket_type,
                    )?;
                }

                // apply the json
                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist {
                    node_index: index.node_index,
                })?;

                println!("found node to apply json");

                node.apply_json(&serialized)?;

                // finally, reinit the node
                graph.graph.init_node(
                    &index.node_index,
                    NodeInitState {
                        props: &HashMap::new(),
                        registry: &mut self.socket_registry,
                        script_engine: &self.scripting_engine,
                        global_state,
                    },
                    false,
                )?;

                Ok(Action::RemoveNode {
                    node_type: Some(node_type),
                    index,
                    child_graph_index,
                    child_graph_io_indexes,
                    connections: Some(connections),
                    serialized: Some(serialized),
                })
            }
            Action::ChangeNodeProperties { index, before, after } => {
                let before = before.ok_or(NodeError::ActionRollbackFieldMissing {
                    missing_field: "before".to_string(),
                })?;

                let mut graph = self.graph_manager.get_graph_wrapper_mut(index.graph_index).ok_or(
                    NodeError::GraphDoesNotExist {
                        graph_index: index.graph_index,
                    },
                )?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist {
                    node_index: index.node_index,
                })?;

                node.set_properties(before.clone());

                graph.graph.init_node(
                    &index.node_index,
                    NodeInitState {
                        props: &HashMap::new(),
                        registry: &mut self.socket_registry,
                        script_engine: &self.scripting_engine,
                        global_state,
                    },
                    false,
                )?;

                action_result.graph_operated_on = Some(index.graph_index);

                Ok(Action::ChangeNodeProperties {
                    index: index,
                    before: Some(before),
                    after: after,
                })
            }
            Action::ChangeNodeUiData { index, before, after } => {
                let before = before.ok_or(NodeError::ActionRollbackFieldMissing {
                    missing_field: "before".to_string(),
                })?;

                let mut graph = self.graph_manager.get_graph_wrapper_mut(index.graph_index).ok_or(
                    NodeError::GraphDoesNotExist {
                        graph_index: index.graph_index,
                    },
                )?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist {
                    node_index: index.node_index,
                })?;

                node.set_ui_data(before.clone());

                action_result.graph_operated_on = Some(index.graph_index);

                if let Some(ref mut to_update) = action_result.defaults_to_update {
                    to_update.push(index.clone())
                } else {
                    action_result.defaults_to_update = Some(vec![index.clone()]);
                }

                Ok(Action::ChangeNodeUiData {
                    index: index,
                    before: Some(before),
                    after: after,
                })
            }
            Action::ChangeNodeOverrides { index, before, after } => {
                let before = before.ok_or(NodeError::ActionRollbackFieldMissing {
                    missing_field: "before".to_string(),
                })?;

                let mut graph = self.graph_manager.get_graph_wrapper_mut(index.graph_index).ok_or(
                    NodeError::GraphDoesNotExist {
                        graph_index: index.graph_index,
                    },
                )?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist {
                    node_index: index.node_index,
                })?;

                node.set_default_overrides(before.clone());

                action_result.graph_operated_on = Some(index.graph_index);

                if let Some(ref mut to_update) = action_result.defaults_to_update {
                    to_update.push(index.clone())
                } else {
                    action_result.defaults_to_update = Some(vec![index.clone()]);
                }

                Ok(Action::ChangeNodeOverrides {
                    index: index,
                    before: Some(before),
                    after: after,
                })
            }
            Action::AddConnection {
                graph_index,
                connection,
            } => {
                let graph = &mut self
                    .graph_manager
                    .get_graph_wrapper_mut(graph_index)
                    .ok_or(NodeError::GraphDoesNotExist {
                        graph_index: graph_index,
                    })?
                    .graph;

                graph.disconnect(
                    &connection.from_node,
                    &connection.from_socket_type,
                    &connection.to_node,
                    &connection.to_socket_type,
                )?;

                action_result.graph_operated_on = Some(graph_index);
                action_result.graph_to_reindex = Some(graph_index);

                Ok(Action::AddConnection {
                    graph_index,
                    connection,
                })
            }
            Action::RemoveConnection {
                graph_index,
                connection,
            } => {
                let graph = &mut self
                    .graph_manager
                    .get_graph_wrapper_mut(graph_index)
                    .ok_or(NodeError::GraphDoesNotExist {
                        graph_index: graph_index,
                    })?
                    .graph;

                graph.connect(
                    &connection.from_node,
                    &connection.from_socket_type,
                    &connection.to_node,
                    &connection.to_socket_type,
                )?;

                action_result.graph_operated_on = Some(graph_index);
                action_result.graph_to_reindex = Some(graph_index);

                Ok(Action::RemoveConnection {
                    graph_index,
                    connection,
                })
            }
        }?;

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
                script_engine: &scripting_engine,
                global_state,
            },
            sound_config,
        )?;

        Ok(())
    }
}
