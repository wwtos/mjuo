use serde::{Serialize, Deserialize};

use crate::{node::Node, nodes::gain_graph_node::GainGraphNode, errors::NodeError};

#[cfg(test)]
use crate::graph_tests::TestNode;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum NodeVariant {
    GainGraphNode(GainGraphNode),
    #[cfg(test)]
    TestNode(TestNode)
}

pub fn new_variant(node_type: String) -> Result<NodeVariant, NodeError> {
    match node_type.as_str() {
        "GainGraphNode" => Ok(NodeVariant::GainGraphNode(GainGraphNode::default())),
        #[cfg(test)]
        "TestNode" => Ok(NodeVariant::TestNode(TestNode::default())),
        _ => Err(NodeError::NodeTypeDoesNotExist)
    }
}

impl<'a> AsRef<dyn Node + 'a> for NodeVariant {
    fn as_ref(&self) -> &(dyn Node + 'a) {
        match self {
            Self::GainGraphNode(node) => node as &dyn Node,
            #[cfg(test)]
            Self::TestNode(node) => node as &dyn Node
        }
    }
}

impl<'a> AsMut<dyn Node + 'a> for NodeVariant {
    fn as_mut(&mut self) -> &mut (dyn Node + 'a) {
        match self {
            Self::GainGraphNode(node) => node as &mut dyn Node,
            #[cfg(test)]
            Self::TestNode(node) => node as &mut dyn Node
        }
    }
}
