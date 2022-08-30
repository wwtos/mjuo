use std::collections::HashMap;

use serde_json::Value;
use sound_engine::SoundConfig;

use crate::{
    connection::Connection,
    errors::NodeError,
    graph_manager::{GlobalNodeIndex, GraphIndex, GraphManager},
    node::{NodeIndex, NodeRow},
    property::Property,
    socket_registry::SocketRegistry,
};
use rhai::Engine;

#[derive(Clone)]
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
        connection: Connection,
    },
    RemoveConnection {
        connection: Connection,
    },
}

pub struct ActionBundle {
    actions: Vec<Action>,
}

pub struct StateManager {
    history: Vec<ActionBundle>,
    graph_manager: GraphManager,
    sound_config: SoundConfig,
    socket_registry: SocketRegistry,
    scripting_engine: Engine,
}

impl StateManager {
    pub fn get_sound_config(&self) -> &SoundConfig {
        &self.sound_config
    }

    pub fn get_registry_and_engine(&mut self) -> (&mut SocketRegistry, &mut Engine) {
        (&mut self.socket_registry, &mut self.scripting_engine)
    }

    fn apply_action(&mut self, action: Action) -> Result<Action, NodeError> {
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
            Action::ChangeNodeProperties { index, before: _, after } => {
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
            Action::ChangeNodeUiData { index, before: _, after } => {
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
            },
            Action::ChangeNodeOverrides { index, before: _, after } => {
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
            },
            Action::AddConnection { connection } => todo!(),
            Action::RemoveConnection { connection } => todo!(),
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
                let connections = connections.ok_or(NodeError::ActionRollbackFieldMissing("connections".to_string()))?;
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
                let before = before
                    .ok_or(NodeError::ActionRollbackFieldMissing("before".to_string()))?;

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
            },
            Action::ChangeNodeUiData { index, before, after } => {
                let before = before
                    .ok_or(NodeError::ActionRollbackFieldMissing("before".to_string()))?;

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
            },
            Action::ChangeNodeOverrides { index, before, after } => {
                let before = before
                    .ok_or(NodeError::ActionRollbackFieldMissing("before".to_string()))?;

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
            },
            Action::AddConnection { connection } => todo!(),
            Action::RemoveConnection { connection } => todo!(),
        }
    }
}
