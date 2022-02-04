use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;

use crate::{node::Node, nodes::gain_graph_node::GainGraphNode};

lazy_static! {
    static ref DEFAULTS: Vec<&'static str> = {
        vec!["GainGraphNode"]
    };
}

#[derive(Serialize, Deserialize, Debug)]
pub enum NodeVariant {
    GainGraphNode(GainGraphNode)
}

impl<'a> AsRef<dyn Node + 'a> for NodeVariant {
    fn as_ref(&self) -> &(dyn Node + 'a) {
        match self {
            Self::GainGraphNode(node) => node as &dyn Node,
        }
    }
}

impl<'a> AsMut<dyn Node + 'a> for NodeVariant {
    fn as_mut(&mut self) -> &mut (dyn Node + 'a) {
        match self {
            Self::GainGraphNode(node) => node as &mut dyn Node,
        }
    }
}
