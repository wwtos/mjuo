use std::collections::HashMap;

use crate::{graph::{PossibleNode}, connection::Connection};
use petgraph::prelude::*;
use petgraph::algo::{greedy_feedback_arc_set, toposort};

pub fn calculate_graph_traverse_order(original_graph: &crate::graph::Graph)-> Vec<crate::node::NodeIndex> {
    // traverse backward and build a list of traverse order

    let mut graph = StableGraph::<usize, Connection>::new();
    let mut graph_lookup: HashMap<usize, NodeIndex> = HashMap::new();

    for (i, original_node) in original_graph.get_nodes().iter().enumerate() {
        match original_node {
            PossibleNode::Some(_) => {
                graph_lookup.insert(i, graph.add_node(i));
            },
            PossibleNode::None(_) => {}
        }
    }

    for original_node in original_graph.get_nodes().iter() {
        match original_node {
            PossibleNode::Some(node) => {
                let node = node.node.borrow();
                let node_index = node.get_index();

                for input in node.list_input_sockets() {
                    graph.add_edge(
                        *graph_lookup.get(&input.from_node.index).unwrap(),
                        *graph_lookup.get(& node.get_index().index).unwrap(),
                        Connection {
                            from_socket_type: input.from_socket_type,
                            from_node: input.from_node,
                            to_socket_type: input.to_socket_type,
                            to_node: node_index
                        }
                    );
                }

                
            },
            PossibleNode::None(_) => {}
        }
    }

    let edges = greedy_feedback_arc_set(&graph);

    let mut connections: Vec<Connection> = Vec::new();
    let mut edge_indexes: Vec<EdgeIndex> = Vec::new();
    
    for edge in edges {
        connections.push(edge.weight().clone());
        edge_indexes.push(edge.id());
    }

    for edge in edge_indexes {
        graph.remove_edge(edge);
    }

    let node_order = toposort(&graph, None).unwrap();

    node_order.iter().map(|index| {
        crate::node::NodeIndex {
            index: index.index(),
            generation: match &original_graph.get_nodes()[index.index()] {
                PossibleNode::Some(node) => node.generation,
                PossibleNode::None(_) => panic!("unreachable")
            }
        }
    }).collect::<Vec<crate::node::NodeIndex>>()
}