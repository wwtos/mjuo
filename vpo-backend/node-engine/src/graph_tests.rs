use ddgg::GraphError;

use crate::connection::{Socket, SocketType};
use crate::errors::NodeError;
use crate::node_graph::NodeGraph;

#[test]
fn graph_node_crud() {
    let mut graph = NodeGraph::new();

    // add a new node
    let (first_node_index, _) = graph.add_node("TestNode".into(), 1).unwrap().value;

    // check that the node exists
    assert!(graph.get_node(first_node_index).is_ok());

    // now let's remove it
    assert!(graph.remove_node(first_node_index).is_ok());

    // let's try removing it twice
    assert_eq!(
        std::mem::discriminant(&graph.remove_node(first_node_index).unwrap_err()),
        std::mem::discriminant(&NodeError::GraphError {
            error: GraphError::VertexDoesNotExist {
                index: first_node_index.0
            }
        })
    );

    // let's try to get it with its index
    assert!(graph.get_node(first_node_index).is_err());

    // now add a second node
    let (second_node_index, _) = graph.add_node("TestNode".into(), 1).unwrap().value;

    // as it took the place of the first one, let's make sure we can't try to
    // retrieve the old one and get the new one
    assert!(graph.get_node(first_node_index).is_err());

    // let's see what happens if we try to delete node one
    assert_eq!(
        format!("{:?}", &graph.remove_node(first_node_index).unwrap_err()),
        format!(
            "{:?}",
            &NodeError::GraphError {
                error: GraphError::VertexDoesNotExist {
                    index: first_node_index.0
                }
            }
        )
    );

    // second node should still exist though with the right generation
    assert!(graph.get_node(second_node_index).is_ok());

    // add another node for good measure to make sure it's growing
    graph.add_node("TestNode".into(), 1).unwrap().value;

    assert_eq!(graph.len(), 2);

    println!("{:?}", graph);
}

#[test]
fn graph_connecting() {
    let mut graph = NodeGraph::new();

    // add two new nodes
    let (first_node_index, _) = graph.add_node("TestNode".into(), 1).unwrap().value;
    let (second_node_index, _) = graph.add_node("TestNode".into(), 1).unwrap().value;
    let (third_node_index, _) = graph.add_node("TestNode".into(), 1).unwrap().value;

    // try connecting the first node to the second node with a socket
    // the the first one doesn't have
    let from_node = graph.get_node(first_node_index).unwrap();

    assert_eq!(
        from_node.has_output_socket(&Socket::Simple("midi".into(), SocketType::Midi, 1)),
        false
    );

    assert_eq!(
        format!(
            "{:?}",
            graph
                .connect(
                    first_node_index,
                    &Socket::Simple("midi".into(), SocketType::Midi, 1),
                    second_node_index,
                    &Socket::Simple("midi".into(), SocketType::Midi, 1),
                )
                .unwrap_err()
        ),
        format!(
            "{:?}",
            NodeError::SocketDoesNotExist {
                socket: Socket::Simple("midi".into(), SocketType::Midi, 1),
            }
        )
    );

    // ditto with on the to side
    let to_node = graph.get_node(first_node_index).unwrap();

    assert_eq!(
        to_node.has_input_socket(&Socket::Simple("nonexistant".into(), SocketType::Stream, 1)),
        false
    );

    assert_eq!(
        format!(
            "{:?}",
            graph
                .connect(
                    first_node_index,
                    &Socket::Simple("audio".into(), SocketType::Stream, 1),
                    second_node_index,
                    &Socket::Simple("nonexistant".into(), SocketType::Stream, 1),
                )
                .unwrap_err()
        ),
        format!(
            "{:?}",
            NodeError::SocketDoesNotExist {
                socket: Socket::Simple("nonexistant".into(), SocketType::Stream, 1)
            }
        )
    );

    // make sure we can't connect two different families of types (midi can't connect to audio, etc)
    assert_eq!(
        format!(
            "{:?}",
            graph
                .connect(
                    first_node_index,
                    &Socket::Simple("audio".into(), SocketType::Stream, 1),
                    second_node_index,
                    &Socket::Simple("midi".into(), SocketType::Midi, 1),
                )
                .unwrap_err()
        ),
        format!(
            "{:?}",
            NodeError::IncompatibleSocketTypes {
                from: SocketType::Stream,
                to: SocketType::Midi
            }
        )
    );

    // but we should be able to connect within the same family
    assert_eq!(
        graph
            .connect(
                first_node_index,
                &Socket::Simple("audio".into(), SocketType::Stream, 1),
                second_node_index,
                &Socket::Simple("audio".into(), SocketType::Stream, 1),
            )
            .is_ok(),
        true
    );

    // but we can't connect twice
    assert_eq!(
        format!(
            "{:?}",
            graph
                .connect(
                    first_node_index,
                    &Socket::Simple("audio".into(), SocketType::Stream, 1),
                    second_node_index,
                    &Socket::Simple("audio".into(), SocketType::Stream, 1),
                )
                .unwrap_err()
        ),
        format!(
            "{:?}",
            NodeError::AlreadyConnected {
                from: Socket::Simple("audio".into(), SocketType::Stream, 1),
                to: Socket::Simple("audio".into(), SocketType::Stream, 1)
            }
        )
    );

    // nor can we connect multiple outputs to one input
    graph
        .connect(
            third_node_index,
            &Socket::Simple("audio".into(), SocketType::Stream, 1),
            second_node_index,
            &Socket::Simple("audio".into(), SocketType::Stream, 1),
        )
        .unwrap_err();

    // but we can connect one output to multiple inputs
    graph
        .connect(
            third_node_index,
            &Socket::Simple("audio".into(), SocketType::Stream, 1),
            second_node_index,
            &Socket::Simple("gain".into(), SocketType::Stream, 1),
        )
        .unwrap();
}

/// This test makes sure that when removing a node, it also removes any
/// connections from all the nodes it's connected to
#[test]
fn hanging_connections() -> Result<(), NodeError> {
    let mut graph = NodeGraph::new();

    // set up a simple network
    let (first_node, _) = graph.add_node("TestNode".into(), 1).unwrap().value;
    let (second_node, _) = graph.add_node("TestNode".into(), 1).unwrap().value;

    graph.connect(
        first_node,
        &Socket::Simple("audio".into(), SocketType::Stream, 1),
        second_node,
        &Socket::Simple("audio".into(), SocketType::Stream, 1),
    )?;

    assert_eq!(graph.get_output_side_connections(first_node)?.len(), 1); // it should be connected here

    graph.remove_node(second_node)?;

    assert_eq!(graph.get_output_side_connections(first_node)?.len(), 0); // it shouldn't be connected to anything

    Ok(())
}
