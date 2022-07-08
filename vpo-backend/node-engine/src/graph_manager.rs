use std::ops::{Index, IndexMut};

use petgraph::{stable_graph::StableDiGraph, graph::NodeIndex};

use crate::node_graph::NodeGraph;

#[derive(Hash, PartialEq, Eq)]
pub struct GraphIndex {
    index: u64,
}

#[derive(Default)]
pub struct GraphManager {
    graphs: StableDiGraph<NodeGraph, ()>,
}

impl GraphManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_graph(&mut self) -> NodeIndex {
        self.graphs.add_node(NodeGraph::new())
    }

    pub fn get_graph_ref(&self, index: NodeIndex) -> Option<&NodeGraph> {
        if self.graphs.contains_node(index) {
            Some(self.graphs.index(index))
        } else {
            None
        }
    }

    pub fn get_graph_mut(&mut self, index: NodeIndex) -> Option<&mut NodeGraph> {
        if self.graphs.contains_node(index) {
            Some(self.graphs.index_mut(index))
        } else {
            None
        }
    }
}
