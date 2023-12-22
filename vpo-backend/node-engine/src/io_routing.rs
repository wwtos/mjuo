use crate::node::NodeIndex;

#[derive(Debug, Clone)]
pub enum DeviceType {
    Midi,
    Stream,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceDirection {
    Source,
    Sink,
}

#[derive(Debug, Clone)]
pub struct RouteRule {
    pub device_id: String,
    pub device_type: DeviceType,
    pub device_direction: DeviceDirection,
    pub device_channel: usize,
    pub node: NodeIndex,
    pub node_socket: usize,
    pub node_channel: usize,
}

#[derive(Debug, Clone)]
pub struct IoRoutes {
    pub rules: Vec<RouteRule>,
}
