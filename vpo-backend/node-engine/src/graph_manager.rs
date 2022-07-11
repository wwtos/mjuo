use std::{collections::HashMap, cell::Ref};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use crate::{node::NodeIndex, node_graph::NodeGraph};

type GraphIndex = u64;

#[derive(Debug)]
pub struct NodeGraphWrapper {
    graph: Rc<RefCell<NodeGraph>>,
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
                graph: Rc::new(RefCell::new(NodeGraph::new())),
                associated_nodes: vec![],
            },
        );

        graph_index
    }

    pub fn get_graph_ref(&self, index: GraphIndex) -> Option<Ref<NodeGraph>> {
        self.node_graphs.get(&index).map(|x| x.graph.borrow())
    }

    pub fn get_graph_mut(&self, index: GraphIndex) -> Option<RefMut<NodeGraph>> {
        self.node_graphs.get_mut(&index).map(|x| x.graph.borrow_mut())
    }

    pub fn get_graph_rc(&self, index: GraphIndex) -> Option<Rc<RefCell<NodeGraph>>> {
        self.node_graphs.get(&index).map(|x| x.graph.clone())
    }
}
