use std::collections::HashMap;

use serde_json::Value;
use sound_engine::{midi::messages::MidiData, SoundConfig};

use crate::{
    connection::{Connection, MidiSocketType, SocketType, StreamSocketType},
    errors::NodeError,
    graph_manager::{GlobalNodeIndex, GraphIndex, GraphManager, NodeGraphWrapper},
    node::{NodeIndex, NodeRow},
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
    },
    RemoveNode {
        node_type: Option<String>,
        index: GlobalNodeIndex,
        child_graph_index: Option<GraphIndex>,
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

#[derive(Clone)]
pub struct ActionBundle {
    actions: Vec<Action>,
}

impl ActionBundle {
    pub fn new(actions: Vec<Action>) -> ActionBundle {
        ActionBundle { actions: actions }
    }
}

pub struct StateManager {
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

impl StateManager {
    pub fn new(sound_config: SoundConfig) -> StateManager {
        let history = Vec::new();
        let place_in_history = 0;
        let mut graph_manager = GraphManager::new();
        let mut socket_registry = SocketRegistry::new();
        let scripting_engine = Engine::new_raw();

        SocketType::register_defaults(&mut socket_registry);

        let root_graph_index = graph_manager.new_graph();

        let (output_node, midi_in_node) = {
            let graph = &mut graph_manager.get_graph_wrapper_mut(root_graph_index).unwrap().graph;

            let output_node = graph.add_node(
                NodeVariant::OutputNode(OutputNode::default()),
                &mut socket_registry,
                &scripting_engine,
            );
            let midi_in_node = graph.add_node(
                NodeVariant::MidiInNode(MidiInNode::default()),
                &mut socket_registry,
                &scripting_engine,
            );

            (output_node, midi_in_node)
        };

        graph_manager
            .recalculate_traversal_for_graph(&root_graph_index)
            .unwrap();

        StateManager {
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
                    .ok_or(NodeError::GraphDoesNotExist(*graph_index))?
                    .graph;
                let subgraph = &mut self
                    .graph_manager
                    .get_graph_wrapper_mut(*graph_index)
                    .ok_or(NodeError::GraphDoesNotExist(*graph_index))?
                    .graph;

                let node = parent_node_graph
                    .get_node_mut(&parent_node_index)
                    .ok_or(NodeError::NodeDoesNotExist(parent_node_index))?;
                node.node_init_graph(subgraph);
            }
        }

        Ok(())
    }

    pub fn step(&mut self, current_time: i64, is_first_time: bool, midi_in: Vec<MidiData>) -> f32 {
        let NodeGraphWrapper {
            ref mut graph,
            ref traverser,
            ..
        } = &mut *self.graph_manager.get_graph_wrapper_mut(self.root_graph_index).unwrap();

        if !midi_in.is_empty() {
            let midi_in_node = graph.get_node_mut(&self.midi_in_node).unwrap();
            midi_in_node.accept_midi_input(&MidiSocketType::Default, midi_in);
        } else if is_first_time {
            let midi_in_node = graph.get_node_mut(&self.midi_in_node).unwrap();
            midi_in_node.accept_midi_input(&MidiSocketType::Default, Vec::new());
        }

        let traversal_errors = traverser.traverse(graph, is_first_time, current_time, &self.scripting_engine);

        if let Err(errors) = traversal_errors {
            println!("{:?}", errors);
        }

        let output_node = graph.get_node_mut(&self.output_node).unwrap();
        let audio = output_node.get_stream_output(&StreamSocketType::Audio);

        audio
    }
}

impl StateManager {
    pub fn commit(&mut self, actions: ActionBundle) -> Result<(), NodeError> {
        let new_actions = actions
            .actions
            .into_iter()
            .map(|action| self.apply_action(action))
            .collect::<Result<Vec<Action>, NodeError>>()?;

        if self.place_in_history < self.history.len() {
            self.history.truncate(self.place_in_history);
        }

        self.history.push(ActionBundle { actions: new_actions });

        self.place_in_history += 1;

        Ok(())
    }

    pub fn undo(&mut self) -> Result<bool, NodeError> {
        if self.place_in_history > 0 {
            let to_rollback = self.history[self.place_in_history].clone();

            // roll back in reverse order
            for action in to_rollback.actions.into_iter().rev() {
                self.rollback_action(action)?;
            }

            self.place_in_history -= 1;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn redo(&mut self) -> Result<bool, NodeError> {
        if self.place_in_history < self.history.len() {
            let to_redo = self.history[self.place_in_history].clone();

            for action in to_redo.actions.into_iter() {
                self.apply_action(action)?;
            }

            self.place_in_history += 1;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn apply_action(&mut self, action: Action) -> Result<Action, NodeError> {
        println!("Applying action: {:?}", action);

        match action {
            Action::CreateNode {
                node_type,
                graph_index,
                node_index,
                child_graph_index: inner_graph_index,
            } => self.graph_manager.create_node_at_index(
                &node_type,
                graph_index,
                node_index,
                inner_graph_index,
                &self.sound_config,
                &mut self.socket_registry,
                &self.scripting_engine,
            ),
            Action::RemoveNode { index, .. } => self.graph_manager.remove_node(&index),
            Action::ChangeNodeProperties {
                index,
                before: _,
                after,
            } => {
                let mut graph = self
                    .graph_manager
                    .get_graph_wrapper_mut(index.graph_index)
                    .ok_or(NodeError::GraphDoesNotExist(index.graph_index))?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist(index.node_index))?;

                let before = node.replace_properties(after.clone());

                Ok(Action::ChangeNodeProperties {
                    index: index,
                    before: Some(before),
                    after: after,
                })
            }
            Action::ChangeNodeUiData {
                index,
                before: _,
                after,
            } => {
                let mut graph = self
                    .graph_manager
                    .get_graph_wrapper_mut(index.graph_index)
                    .ok_or(NodeError::GraphDoesNotExist(index.graph_index))?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist(index.node_index))?;

                let before = node.replace_ui_data(after.clone());

                Ok(Action::ChangeNodeUiData {
                    index: index,
                    before: Some(before),
                    after: after,
                })
            }
            Action::ChangeNodeOverrides {
                index,
                before: _,
                after,
            } => {
                let mut graph = self
                    .graph_manager
                    .get_graph_wrapper_mut(index.graph_index)
                    .ok_or(NodeError::GraphDoesNotExist(index.graph_index))?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist(index.node_index))?;

                let before = node.replace_default_overrides(after.clone());

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
                    .ok_or(NodeError::GraphDoesNotExist(graph_index))?
                    .graph;

                graph.connect(
                    &connection.from_node,
                    &connection.from_socket_type,
                    &connection.to_node,
                    &connection.to_socket_type,
                )?;

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
                    .ok_or(NodeError::GraphDoesNotExist(graph_index))?
                    .graph;

                graph.disconnect(
                    &connection.from_node,
                    &connection.from_socket_type,
                    &connection.to_node,
                    &connection.to_socket_type,
                )?;

                Ok(Action::RemoveConnection {
                    graph_index,
                    connection,
                })
            }
        }
    }

    fn rollback_action(&mut self, action: Action) -> Result<Action, NodeError> {
        match action {
            Action::CreateNode {
                node_type,
                graph_index,
                node_index,
                child_graph_index: inner_graph_index,
            } => {
                let node_index = node_index.ok_or(NodeError::ActionRollbackFieldMissing("node_index".to_string()))?;

                self.graph_manager
                    .remove_node(&GlobalNodeIndex {
                        graph_index: graph_index,
                        node_index: node_index.clone(),
                    })
                    .map(|_| Action::CreateNode {
                        node_type: node_type.clone(),
                        graph_index: graph_index.clone(),
                        node_index: Some(node_index),
                        child_graph_index: inner_graph_index.clone(),
                    })
            }
            Action::RemoveNode {
                node_type,
                index,
                child_graph_index,
                connections,
                serialized,
            } => {
                // unwrap all the Option fields (should be present for a rollback)
                let node_type = node_type.ok_or(NodeError::ActionRollbackFieldMissing("node_type".to_string()))?;
                let connections =
                    connections.ok_or(NodeError::ActionRollbackFieldMissing("connections".to_string()))?;
                let serialized = serialized.ok_or(NodeError::ActionRollbackFieldMissing("serialized".to_string()))?;

                self.graph_manager.create_node_at_index(
                    &node_type,
                    index.graph_index,
                    Some(index.node_index),
                    child_graph_index,
                    &self.sound_config,
                    &mut self.socket_registry,
                    &self.scripting_engine,
                )?;

                // connect everything back up
                let mut graph = self
                    .graph_manager
                    .get_graph_wrapper_mut(index.graph_index)
                    .ok_or(NodeError::GraphDoesNotExist(index.graph_index))?;

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
                let node = node.ok_or(NodeError::NodeDoesNotExist(index.node_index))?;

                node.apply_json(&serialized)?;

                // finally, reinit the node
                graph
                    .graph
                    .init_node(&index.node_index, &mut self.socket_registry, &self.scripting_engine)?;

                Ok(Action::RemoveNode {
                    node_type: Some(node_type),
                    index,
                    child_graph_index,
                    connections: Some(connections),
                    serialized: Some(serialized),
                })
            }
            Action::ChangeNodeProperties { index, before, after } => {
                let before = before.ok_or(NodeError::ActionRollbackFieldMissing("before".to_string()))?;

                let mut graph = self
                    .graph_manager
                    .get_graph_wrapper_mut(index.graph_index)
                    .ok_or(NodeError::GraphDoesNotExist(index.graph_index))?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist(index.node_index))?;

                node.set_properties(before.clone());

                Ok(Action::ChangeNodeProperties {
                    index: index,
                    before: Some(before),
                    after: after,
                })
            }
            Action::ChangeNodeUiData { index, before, after } => {
                let before = before.ok_or(NodeError::ActionRollbackFieldMissing("before".to_string()))?;

                let mut graph = self
                    .graph_manager
                    .get_graph_wrapper_mut(index.graph_index)
                    .ok_or(NodeError::GraphDoesNotExist(index.graph_index))?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist(index.node_index))?;

                node.set_ui_data(before.clone());

                Ok(Action::ChangeNodeUiData {
                    index: index,
                    before: Some(before),
                    after: after,
                })
            }
            Action::ChangeNodeOverrides { index, before, after } => {
                let before = before.ok_or(NodeError::ActionRollbackFieldMissing("before".to_string()))?;

                let mut graph = self
                    .graph_manager
                    .get_graph_wrapper_mut(index.graph_index)
                    .ok_or(NodeError::GraphDoesNotExist(index.graph_index))?;

                let node = graph.graph.get_node_mut(&index.node_index);
                let node = node.ok_or(NodeError::NodeDoesNotExist(index.node_index))?;

                node.set_default_overrides(before.clone());

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
                    .ok_or(NodeError::GraphDoesNotExist(graph_index))?
                    .graph;

                graph.disconnect(
                    &connection.from_node,
                    &connection.from_socket_type,
                    &connection.to_node,
                    &connection.to_socket_type,
                )?;

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
                    .ok_or(NodeError::GraphDoesNotExist(graph_index))?
                    .graph;

                graph.connect(
                    &connection.from_node,
                    &connection.from_socket_type,
                    &connection.to_node,
                    &connection.to_socket_type,
                )?;

                Ok(Action::RemoveConnection {
                    graph_index,
                    connection,
                })
            }
        }
    }
}
