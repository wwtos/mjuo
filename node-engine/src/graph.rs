use crate::{
    errors::Error,
    node::{Connection, ConnectionType, GenerationalNode, Node, NodeIndex},
};

pub struct Graph {
    nodes: Vec<GenerationalNode>,
}

impl Graph {
    

    pub fn get_node(&mut self, index: NodeIndex) -> Option<GenerationalNode> {
        Some(self.nodes[index.index].clone())
    }
}
