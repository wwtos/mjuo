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

    pub fn apply_action(&mut self, action: &Action) -> Result<Action, NodeError> {
        match action {
            Action::CreateNode {
                node_type,
                graph_index,
                node_index,
                child_graph_index: inner_graph_index,
            } => self.graph_manager.create_node_unchecked(
                &node_type,
                *graph_index,
                *node_index,
                *inner_graph_index,
                &self.sound_config,
                &mut self.socket_registry,
                &self.scripting_engine,
            ),
            Action::RemoveNode { index, .. } => self.graph_manager.remove_node(index),
            Action::ChangeNodeProperties { index, before, after } => todo!(),
            Action::ChangeNodeUiData { index, before, after } => todo!(),
            Action::ChangeNodeOverrides { index, before, after } => todo!(),
            Action::AddConnection { connection } => todo!(),
            Action::RemoveConnection { connection } => todo!(),
        }
    }

    pub fn rollback_action(&mut self, action: &Action) -> Result<Action, NodeError> {
        match action {
            Action::CreateNode {
                node_type,
                graph_index,
                node_index,
                child_graph_index: inner_graph_index,
            } => {
                let node_index = node_index.expect("node_index never initialized when action was first done!");

                self.graph_manager
                    .remove_node(&GlobalNodeIndex {
                        graph_index: *graph_index,
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
                let node_type = node_type
                    .as_ref()
                    .ok_or(NodeError::ActionRollbackFieldMissing("node_type".to_string()))?;
                let connections = connections
                    .as_ref()
                    .ok_or(NodeError::ActionRollbackFieldMissing("connections".to_string()))?;
                let serialized = serialized
                    .as_ref()
                    .ok_or(NodeError::ActionRollbackFieldMissing("serialized".to_string()))?;

                self.graph_manager.create_node_unchecked(
                    &node_type,
                    index.graph_index,
                    Some(index.node_index),
                    *child_graph_index,
                    &self.sound_config,
                    &mut self.socket_registry,
                    &self.scripting_engine,
                )?;

                // connect everything back up
                let mut graph = self
                    .graph_manager
                    .get_graph_wrapper_mut(index.graph_index)
                    .ok_or(NodeError::GraphDoesNotExist(index.graph_index))?;

                for connection in connections {
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

                // finally, reinit it
                graph
                    .graph
                    .init_node(&index.node_index, &mut self.socket_registry, &self.scripting_engine)?;

                Ok(action.clone())
            }
            Action::ChangeNodeProperties { index, before, after } => todo!(),
            Action::ChangeNodeUiData { index, before, after } => todo!(),
            Action::ChangeNodeOverrides { index, before, after } => todo!(),
            Action::AddConnection { connection } => todo!(),
            Action::RemoveConnection { connection } => todo!(),
        }
    }
}
