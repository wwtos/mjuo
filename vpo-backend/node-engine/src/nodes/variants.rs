use serde::{Serialize, Deserialize};

use crate::{node::Node, nodes::gain_graph_node::GainGraphNode};

#[cfg(test)]
use crate::graph_tests::TestNode;

#[derive(Serialize, Deserialize, Debug)]
pub enum NodeVariant {
    GainGraphNode(GainGraphNode),
    #[cfg(test)]
    TestNode(TestNode)
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