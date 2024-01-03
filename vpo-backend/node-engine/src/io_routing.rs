use serde::{Deserialize, Serialize};

use crate::node::NodeIndex;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "variant", content = "data")]
pub enum DeviceType {
    Midi,
    Stream,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "variant", content = "data")]
pub enum DeviceDirection {
    Source,
    Sink,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct RouteRule {
    pub device_id: String,
    pub device_type: DeviceType,
    pub device_direction: DeviceDirection,
    pub device_channel: usize,
    pub node: NodeIndex,
    pub node_socket: usize,
    pub node_channel: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub name: String,
    pub device_type: DeviceType,
    pub device_direction: DeviceDirection,
    pub channels: usize,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IoRoutes {
    pub rules: Vec<RouteRule>,
    pub devices: Vec<DeviceInfo>,
}
