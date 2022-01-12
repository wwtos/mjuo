use serde_json::Value;

use crate::{node::Node, nodes::gain_graph_node::GainGraphNode};

pub fn deserialize_by_type(node_type: &str, json: Value) -> Box<dyn Node> {
    match node_type {
        "GainGraphNode" => Box::new(GainGraphNode::deserialize_from_json(json)),
        _ => panic!("not available type"),
    }
}
