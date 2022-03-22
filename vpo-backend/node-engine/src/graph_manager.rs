use std::collections::HashMap;

use crate::graph::Graph;

#[derive(Hash, PartialEq, Eq)]
pub struct GraphIndex {
    index: u64,
}

#[derive(Default)]
pub struct GraphManager {
    graphs: HashMap<GraphIndex, Graph>,
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
            Graph::new(),
        );

        GraphIndex {
            index: self.graph_id_counter - 1,
        }
    }

    pub fn get_graph_ref(&self, index: &GraphIndex) -> Option<&Graph> {
        self.graphs.get(index)
    }

    pub fn get_graph_mut(&mut self, index: &GraphIndex) -> Option<&mut Graph> {
        self.graphs.get_mut(index)
    }
}
