use std::borrow::Borrow;
use std::ptr::NonNull;

use serde_json;

use crate::errors::{Error, ErrorType};
use crate::connection::{MidiSocketType, SocketType, StreamSocketType, ValueType};
use crate::{graph::Graph, node::Node};

#[derive(Debug)]
struct TestNode {}

impl Node for TestNode {
    fn list_input_sockets(&self) -> Vec<SocketType> {
        vec![
            SocketType::Stream(StreamSocketType::Audio),
            SocketType::Stream(StreamSocketType::Detune),
            SocketType::Midi(MidiSocketType::Default),
        ]
    }

    fn list_output_sockets(&self) -> Vec<SocketType> {
        vec![
            SocketType::Stream(StreamSocketType::Audio),
            SocketType::Value(ValueType::Gain),
        ]
    }

    fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32) {}
    fn get_stream_output(&mut self, socket_type: StreamSocketType) -> f32 {
        0_f32
    }

    fn serialize_to_json(&self) -> Result<serde_json::Value, Error> {
        Ok(serde_json::Value::Null)
    }

    fn deserialize_from_json(json: serde_json::Value) -> Self
    where
        Self: Sized,
    {
        TestNode {}
    }
}

#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn graph_node_crud() {
    let mut graph = Graph::new();

    // add a new node
    let first_node_index = graph.add_node(Box::new(TestNode {}));

    // check that the node exists
    assert!(graph.get_node(&first_node_index).is_some());

    // now let's remove it
    assert_eq!(graph.remove_node(&first_node_index), Ok(()));

    // let's try removing it twice
    assert_eq!(
        graph.remove_node(&first_node_index).unwrap_err().error_type,
        ErrorType::NodeDoesNotExist
    );

    // let's try to get it with its index
    assert!(graph.get_node(&first_node_index).is_none());

    // now add a second node
    let second_node_index = graph.add_node(Box::new(TestNode {}));

    // it should have taken the place of the first node
    assert_eq!(first_node_index.index, second_node_index.index);

    // as it took the place of the first one, let's make sure we can't try to
    // retrieve the old one and get the new one
    assert!(graph.get_node(&first_node_index).is_none());

    // let's see what happens if we try to delete node one
    assert_eq!(
        graph.remove_node(&first_node_index).unwrap_err().error_type,
        ErrorType::NodeDoesNotExist
    );

    // second node should still exist though with the right generation
    assert!(graph.get_node(&second_node_index).is_some());

    // add another node for good measure to make sure it's growing
    graph.add_node(Box::new(TestNode {}));
    assert_eq!(graph.len(), 2);

    println!("{:?}", graph.serialize());
}

#[test]
fn graph_connecting() {
    let mut graph = Graph::new();

    // add two new nodes
    let first_node_index = graph.add_node(Box::new(TestNode {}));
    let second_node_index = graph.add_node(Box::new(TestNode {}));

    // try connecting the first node to the second node with a socket
    // the the first one doesn't have
    {
        let from_node_wrapped = graph.get_node(&first_node_index).unwrap().node;
        let from_node = (*from_node_wrapped).borrow();

        assert_eq!(
            from_node.has_output_socket(&SocketType::Midi(MidiSocketType::Default)),
            false
        );
        // drop `from` node borrow
    }

    assert_eq!(
        graph
            .connect(
                first_node_index,
                SocketType::Midi(MidiSocketType::Default),
                second_node_index,
                SocketType::Midi(MidiSocketType::Default),
            )
            .unwrap_err()
            .error_type,
        ErrorType::SocketDoesNotExist
    );

    // ditto with on the to side
    {
        let to_node_wrapped = graph.get_node(&first_node_index).unwrap().node;
        let to_node = (*to_node_wrapped).borrow();

        assert_eq!(
            to_node.has_input_socket(&SocketType::Stream(StreamSocketType::Dynamic(2))),
            false
        );
        // drop `to` node borrow
    }

    assert_eq!(
        graph
            .connect(
                first_node_index,
                SocketType::Stream(StreamSocketType::Audio),
                second_node_index,
                SocketType::Stream(StreamSocketType::Dynamic(2)),
            )
            .unwrap_err()
            .error_type,
        ErrorType::SocketDoesNotExist
    );

    // make sure we can't connect two different families of types (midi can't connect to audio, etc)
    assert_eq!(
        graph
            .connect(
                first_node_index,
                SocketType::Stream(StreamSocketType::Audio),
                second_node_index,
                SocketType::Midi(MidiSocketType::Default),
            )
            .unwrap_err()
            .error_type,
        ErrorType::IncompatibleSocketTypes
    );

    // but we should be able to connect within the same family
    assert_eq!(
        graph
            .connect(
                first_node_index,
                SocketType::Stream(StreamSocketType::Audio),
                second_node_index,
                SocketType::Stream(StreamSocketType::Audio),
            )
            .is_ok(),
        true
    );

    // but we can't connect twice
    assert_eq!(
        graph
            .connect(
                first_node_index,
                SocketType::Stream(StreamSocketType::Audio),
                second_node_index,
                SocketType::Stream(StreamSocketType::Audio),
            )
            .unwrap_err()
            .error_type,
        ErrorType::AlreadyConnected
    );

    // nor can we connect multiple outputs to one input
    assert_eq!(
        graph
            .connect(
                first_node_index,
                SocketType::Stream(StreamSocketType::Detune),
                second_node_index,
                SocketType::Stream(StreamSocketType::Audio),
            )
            .unwrap_err()
            .error_type,
        ErrorType::AlreadyConnected
    );

    // but we can connect one output to multiple inputs
    assert_eq!(
        graph
            .connect(
                first_node_index,
                SocketType::Stream(StreamSocketType::Audio),
                second_node_index,
                SocketType::Stream(StreamSocketType::Detune),
            )
            .is_ok(),
        true
    );
}
