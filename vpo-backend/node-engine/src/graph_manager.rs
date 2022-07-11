use std::collections::HashMap;

use crate::{node::NodeIndex, node_graph::NodeGraph};

type GraphIndex = u64;

#[derive(Debug)]
pub struct NodeGraphWrapper {
    graph: NodeGraph,
    associated_nodes: Vec<NodeIndex>,
}

#[derive(Default, Debug)]
pub struct GraphManager {
    node_graphs: HashMap<u64, NodeGraphWrapper>,
    current_uid: u64,
}

impl GraphManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_graph(&mut self) -> GraphIndex {
        let graph_index = self.current_uid;
        self.current_uid += 1;

        self.node_graphs.insert(
            graph_index,
            NodeGraphWrapper {
                graph: NodeGraph::new(),
                associated_nodes: vec![],
            },
        );

        graph_index
    }

    pub fn get_graph_ref(&self, index: GraphIndex) -> Option<&NodeGraph> {
        self.node_graphs.get(&index).map(|x| &x.graph)
    }

    pub fn get_graph_mut(&mut self, index: GraphIndex) -> Option<&mut NodeGraph> {
        self.node_graphs.get_mut(&index).map(|x| &mut x.graph)
    }
}
