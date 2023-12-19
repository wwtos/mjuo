use std::ops::Range;

use crate::node::NodeIndex;

#[derive(Debug)]
pub enum DeviceType {
    Midi,
    Stream,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeviceDirection {
    Source,
    Sink,
}

#[derive(Debug)]
pub struct RouteRule {
    pub device_id: String,
    pub device_type: DeviceType,
    pub device_direction: DeviceDirection,
    pub device_channels: Range<usize>,
    pub node: NodeIndex,
    pub node_channels: Range<usize>,
}

#[derive(Debug)]
pub struct IoRoutes {
    pub rules: Vec<RouteRule>,
}
