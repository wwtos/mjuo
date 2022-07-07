use std::collections::HashMap;

use crate::node_graph::NodeGraph;

#[derive(Hash, PartialEq, Eq)]
pub struct GraphIndex {
    index: u64,
}

#[derive(Default)]
pub struct GraphManager {
    graphs: HashMap<GraphIndex, NodeGraph>,
    graph_id_counter: u64,
}

impl GraphManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_graph(&mut self) -> GraphIndex {
        self.graph_id_counter += 1;

        self.graphs.insert(
            GraphIndex {
                index: self.graph_id_counter - 1,
            },
            NodeGraph::new(),
        );

        GraphIndex {
            index: self.graph_id_counter - 1,
        }
    }

    pub fn get_graph_ref(&self, index: &GraphIndex) -> Option<&NodeGraph> {
        self.graphs.get(index)
    }

    pub fn get_graph_mut(&mut self, index: &GraphIndex) -> Option<&mut NodeGraph> {
        self.graphs.get_mut(index)
    }
}
