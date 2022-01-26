use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;

use crate::{node::Node, nodes::gain_graph_node::GainGraphNode};

lazy_static! {
    static ref DEFAULTS: Vec<&'static str> = {
        vec!["GainGraphNode"]
    };
}

#[derive(Serialize, Deserialize)]
pub enum NodeVariant {
    GainGraphNode(GainGraphNode)
}

impl From<NodeVariant> for Box<dyn Node> {
    fn from(variant: NodeVariant) -> Self {
        match variant {
            NodeVariant::GainGraphNode(node) => Box::new(node)
        }
    }
}
