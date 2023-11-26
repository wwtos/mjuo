use std::{collections::HashMap, mem};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    connection::{Socket, SocketDirection},
    errors::{NodeError, NodeOk},
    node::{NodeGetIoContext, NodeGraphAndIo, NodeRow, NodeState},
    nodes::variant_io,
    property::Property,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeInstance {
    node_type: String,
    #[serde(skip_deserializing)]
    node_rows: Vec<NodeRow>,
    default_overrides: Vec<NodeRow>,
    properties: HashMap<String, Property>,
    ui_data: HashMap<String, Value>,
    state: NodeState,
    child_graph: Option<NodeGraphAndIo>,
}

impl NodeInstance {
    pub fn new(node_type: String, node_rows: Vec<NodeRow>) -> Result<NodeOk<NodeInstance>, NodeError> {
        // extract properties from result from `init`
        // this fills the properties with the default values
        let properties = node_rows
            .iter()
            .filter_map(|row| match row {
                NodeRow::Property(name, _, default) => Some((name, default)),
                _ => None,
            })
            .fold(HashMap::new(), |mut accum, (name, default)| {
                accum.insert(name.clone(), default.clone());
                accum
            });

        let mut wrapper = NodeInstance {
            node_type: node_type.clone(),
            default_overrides: Vec::new(),
            node_rows,
            properties,
            ui_data: HashMap::new(),
            state: NodeState::default(),
            child_graph: None,
        };

        // insert some initial UI data
        wrapper.ui_data.insert("x".to_string(), json! { 0.0_f32 });
        wrapper.ui_data.insert("y".to_string(), json! { 0.0_f32 });

        wrapper.ui_data.insert("title".to_string(), json! { node_type });

        NodeOk::no_warnings(wrapper)
    }

    pub fn uses_child_graph(&self) -> bool {
        self.node_rows.iter().any(|row| matches!(row, NodeRow::InnerGraph))
    }

    pub fn set_child_graph(&mut self, graph: NodeGraphAndIo) {
        self.child_graph = Some(graph);
    }

    pub fn get_child_graph(&self) -> &Option<NodeGraphAndIo> {
        &self.child_graph
    }

    pub fn get_node_rows(&self) -> &Vec<NodeRow> {
        &self.node_rows
    }

    pub fn set_node_rows(&mut self, rows: Vec<NodeRow>) -> Vec<NodeRow> {
        mem::replace(&mut self.node_rows, rows)
    }

    pub fn refresh_node_rows(&mut self, ctx: &NodeGetIoContext) {
        self.node_rows = variant_io(&self.node_type, ctx, self.properties.clone())
            .unwrap()
            .node_rows;
    }

    pub fn get_property(&self, name: &str) -> Option<Property> {
        self.properties.get(name).cloned()
    }

    pub fn set_property(&mut self, name: String, value: Property) {
        self.properties.insert(name, value);
    }

    pub fn get_properties(&self) -> &HashMap<String, Property> {
        &self.properties
    }

    pub fn set_properties(&mut self, properties: HashMap<String, Property>) -> HashMap<String, Property> {
        mem::replace(&mut self.properties, properties)
    }

    pub fn get_state(&self) -> &NodeState {
        &self.state
    }

    pub fn set_state(&mut self, state: NodeState) -> NodeState {
        mem::replace(&mut self.state, state)
    }

    pub fn get_ui_data(&self) -> &HashMap<String, Value> {
        &self.ui_data
    }

    pub fn set_ui_data(&mut self, ui_data: HashMap<String, Value>) -> HashMap<String, Value> {
        mem::replace(&mut self.ui_data, ui_data)
    }

    pub fn extend_ui_data(&mut self, ui_data: HashMap<String, Value>) {
        self.ui_data.extend(ui_data.into_iter());
    }

    pub fn get_default_overrides(&self) -> &Vec<NodeRow> {
        &self.default_overrides
    }

    pub fn set_default_overrides(&mut self, default_overrides: Vec<NodeRow>) -> Vec<NodeRow> {
        mem::replace(&mut self.default_overrides, default_overrides)
    }

    pub fn set_ui_data_property(&mut self, key: String, value: Value) {
        self.ui_data.insert(key, value);
    }

    /// Guaranteed to be in order based on local node rows
    pub fn list_input_sockets(&self) -> Vec<&Socket> {
        self.node_rows
            .iter()
            .filter_map(|row| {
                let (socket, direction) = row.to_socket_and_direction()?;

                match direction {
                    SocketDirection::Input => Some(socket),
                    SocketDirection::Output => None,
                }
            })
            .collect()
    }

    pub fn list_output_sockets(&self) -> Vec<&Socket> {
        self.node_rows
            .iter()
            .filter_map(|row| {
                let (socket, direction) = row.to_socket_and_direction()?;

                match direction {
                    SocketDirection::Input => None,
                    SocketDirection::Output => Some(socket),
                }
            })
            .collect()
    }

    pub fn has_input_socket(&self, to_find: &Socket) -> bool {
        self.list_input_sockets().iter().any(|&socket| socket == to_find)
    }

    pub fn has_output_socket(&self, to_find: &Socket) -> bool {
        self.list_output_sockets().iter().any(|&socket| socket == to_find)
    }

    pub fn get_default(&self, to_find: &Socket) -> Option<NodeRow> {
        let possible_override = self.default_overrides.iter().find(|override_row| {
            let type_and_direction = override_row.to_socket_and_direction();

            if let Some((override_type, override_direction)) = type_and_direction {
                to_find == override_type && SocketDirection::Input == override_direction
            } else {
                false
            }
        });

        if let Some(row_override) = possible_override {
            return Some(row_override.clone());
        }

        self.node_rows
            .iter()
            .find(|node_row| {
                let type_and_direction = node_row.to_socket_and_direction();

                if let Some((override_type, override_direction)) = type_and_direction {
                    to_find == override_type && SocketDirection::Input == override_direction
                } else {
                    false
                }
            })
            .cloned()
    }

    pub fn get_node_type(&self) -> String {
        self.node_type.clone()
    }
}
