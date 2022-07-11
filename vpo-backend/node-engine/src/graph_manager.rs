use std::{collections::HashMap, cell::Ref};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use crate::traversal::traverser::Traverser;
use crate::{node::NodeIndex, node_graph::NodeGraph};

type GraphIndex = u64;

#[derive(Debug)]
pub struct NodeGraphWrapper {
    graph: NodeGraph,
    traverser: Traverser,
    associated_nodes: Vec<NodeIndex>,
}

#[derive(Default, Debug)]
pub struct GraphManager {
    node_graphs: HashMap<u64, Rc<RefCell<NodeGraphWrapper>>>,
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
            Rc::new(RefCell::new(NodeGraphWrapper {
                graph: NodeGraph::new(),
                traverser: Traverser::default(),
                associated_nodes: vec![],
            })),
        );

        graph_index
    }

    pub fn get_graph_wrapper_ref(&self, index: GraphIndex) -> Option<Ref<NodeGraphWrapper>> {
        self.node_graphs.get(&index).map(|x| (*x).borrow())
    }

    pub fn get_graph_wrapper_mut(&self, index: GraphIndex) -> Option<RefMut<NodeGraphWrapper>> {
        self.node_graphs.get(&index).map(|x| (*x).borrow_mut())
    }

    pub fn get_graph_wrapper_rc(&self, index: GraphIndex) -> Option<Rc<RefCell<NodeGraphWrapper>>> {
        self.node_graphs.get(&index).map(|x| x.clone())
    }
}
