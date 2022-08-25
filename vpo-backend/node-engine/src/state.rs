use std::collections::HashMap;

use async_std::channel::Sender;
use serde_json::Value;
use sound_engine::SoundConfig;

use crate::{
    graph_manager::{GraphManager, GraphIndex, GlobalNodeIndex},
    node::{NodeIndex, NodeRow},
    connection::Connection,
    property::Property,
    socket_registry::SocketRegistry, errors::NodeError
};
use rhai::Engine;

pub enum Action {
    CreateNode {
        node_type: String,
        graph_index: GraphIndex,
        node_index: Option<NodeIndex>,
        inner_graph_index: Option<GraphIndex>
    },
    RemoveNode {
        index: GlobalNodeIndex,
        connections: Vec<Connection>
    },
    ChangeNodeProperties {
        index: GlobalNodeIndex,
        before: HashMap<String, Property>,
        after: HashMap<String, Property>
    },
    ChangeNodeUiData {
        index: GlobalNodeIndex,
        before: HashMap<String, Value>,
        after: HashMap<String, Value>
    },
    ChangeNodeOverrides {
        index: GlobalNodeIndex,
        before: Vec<NodeRow>,
        after: Vec<NodeRow>
    },
    AddConnection {
        connection: Connection
    },
    RemoveConnection {
        connection: Connection
    },
}

pub struct ActionBundle {
    actions: Vec<Action>
}

pub struct StateManager {
    history: Vec<ActionBundle>,
    graph_manager: GraphManager,
    sound_config: SoundConfig,
    socket_registry: SocketRegistry,
    scripting_engine: Engine
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
            Action::CreateNode { node_type, graph_index, node_index, inner_graph_index } => {
                self.graph_manager.create_node_unchecked(
                    &node_type,
                    *graph_index,
                    *node_index,
                    *inner_graph_index,
                    &self.sound_config,
                    &mut self.socket_registry,
                    &self.scripting_engine,
                )
            },
            Action::RemoveNode { index, connections } => todo!(),
            Action::ChangeNodeProperties { index, before, after } => todo!(),
            Action::ChangeNodeUiData { index, before, after } => todo!(),
            Action::ChangeNodeOverrides { index, before, after } => todo!(),
            Action::AddConnection { connection } => todo!(),
            Action::RemoveConnection { connection } => todo!(),
        }
    }

    pub fn rollback_action(&mut self, action: &Action) {
        match action {
            Action::CreateNode { node_type, graph_index, node_index, inner_graph_index } => {
                
            },
            Action::RemoveNode { index, connections } => todo!(),
            Action::ChangeNodeProperties { index, before, after } => todo!(),
            Action::ChangeNodeUiData { index, before, after } => todo!(),
            Action::ChangeNodeOverrides { index, before, after } => todo!(),
            Action::AddConnection { connection } => todo!(),
            Action::RemoveConnection { connection } => todo!(),
        }
    }
}