use std::cell::{Cell, RefCell, RefMut};
use std::rc::Rc;
use std::{cell::Ref, collections::HashMap};

use crate::node_graph::PossibleNode;
use crate::traversal::traverser::Traverser;
use crate::{node::NodeIndex, node_graph::NodeGraph};

pub type GraphIndex = u64;

#[derive(Debug)]
pub struct NodeGraphWrapper {
    pub graph: NodeGraph,
    pub traverser: Traverser,
    associated_nodes: Vec<NodeIndex>,
}

#[derive(Default, Debug)]
pub struct GraphManager {
    node_graphs: HashMap<u64, RefCell<NodeGraphWrapper>>,
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
            RefCell::new(NodeGraphWrapper {
                graph: NodeGraph::new(),
                traverser: Traverser::default(),
                associated_nodes: vec![],
            }),
        );

        graph_index
    }

    pub fn get_graph_wrapper_ref(&self, index: GraphIndex) -> Option<Ref<NodeGraphWrapper>> {
        self.node_graphs.get(&index).map(|x| (*x).borrow())
    }

    pub fn get_graph_wrapper_mut(&self, index: GraphIndex) -> Option<RefMut<NodeGraphWrapper>> {
        self.node_graphs.get(&index).map(|x| (*x).borrow_mut())
    }

    pub fn recalculate_traversal_for_graph(&self, index: GraphIndex) {
        let graph_wrapper = self.get_graph_wrapper_mut(index);

        // set the new traverser
        if let Some(mut graph_wrapper) = graph_wrapper {
            graph_wrapper.traverser = Traverser::get_traverser(&graph_wrapper.graph);
        }
    }

    pub fn update_traversal_defaults(&self, index: GraphIndex, nodes_to_update: Vec<NodeIndex>) {
        let graph_wrapper = self.get_graph_wrapper_mut(index);

        if let Some(mut graph_wrapper) = graph_wrapper {
            let NodeGraphWrapper { traverser, graph, .. } = &mut *graph_wrapper;

            for node_index in nodes_to_update.iter() {
                traverser.update_node_defaults(graph, node_index);
            }
        }
    }
}
