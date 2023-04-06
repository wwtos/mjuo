use std::collections::HashMap;

use crate::connection::Connection;
use crate::node;
use crate::node_graph::NodeConnection;

use ddgg::VertexIndex;
use petgraph::algo::{greedy_feedback_arc_set, toposort};
use petgraph::prelude::*;

pub fn calculate_graph_traverse_order(original_graph: &crate::node_graph::NodeGraph) -> Vec<node::NodeIndex> {
    // traverse backward and build a traversal order

    let mut graph = StableGraph::<node::NodeIndex, NodeConnection>::new();
    let mut graph_lookup: HashMap<VertexIndex, NodeIndex> = HashMap::new();

    for original_node_index in original_graph.node_indexes() {
        graph_lookup.insert(original_node_index.0, graph.add_node(original_node_index));
    }

    for original_edge_index in original_graph.edge_indexes() {
        let edge = original_graph.get_graph().get_edge(original_edge_index.0).unwrap();

        graph.add_edge(
            graph_lookup[&edge.get_from()],
            graph_lookup[&edge.get_to()],
            edge.data.clone(),
        );
    }

    let edges = greedy_feedback_arc_set(&graph);

    let mut connections: Vec<Connection> = Vec::new();
    let mut edge_indexes: Vec<EdgeIndex> = Vec::new();

    for edge in edges {
        let weight = edge.weight();

        connections.push(Connection {
            from_node: graph[edge.source()],
            to_node: graph[edge.target()],
            data: NodeConnection {
                from_socket: weight.from_socket.clone(),
                to_socket: weight.to_socket.clone(),
            },
        });

        edge_indexes.push(edge.id());
    }

    for edge in edge_indexes {
        graph.remove_edge(edge);
    }

    let node_order = toposort(&graph, None).unwrap();

    node_order
        .iter()
        .map(|index| *graph.node_weight(*index).unwrap())
        .collect::<Vec<crate::node::NodeIndex>>()
}
