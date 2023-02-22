use std::collections::HashMap;

use rhai::Engine;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use sound_engine::SoundConfig;

use crate::connection::{MidiSocketType, Primitive, SocketType, StreamSocketType, ValueSocketType};
use crate::errors::{NodeError, NodeOk};
use crate::global_state::GlobalState;
use crate::node::{InitResult, NodeIndex, NodeInitState, NodeRow};
use crate::nodes::variants::NodeVariant;
use crate::socket_registry::SocketRegistry;
use crate::{node::Node, node_graph::NodeGraph};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestNode {}

impl Default for TestNode {
    fn default() -> Self {
        TestNode {}
    }
}

impl Node for TestNode {
    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![
            NodeRow::StreamInput(StreamSocketType::Audio, 0.0, false),
            NodeRow::StreamInput(StreamSocketType::Detune, 0.0, false),
            NodeRow::MidiInput(MidiSocketType::Default, SmallVec::new(), false),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0, false),
            NodeRow::ValueOutput(ValueSocketType::Gain, Primitive::Float(0.0), false),
        ])
    }
}

#[test]
fn graph_node_crud() {
    let mut graph = NodeGraph::new();
    let mut registry = SocketRegistry::new();
    let scripting_engine = Engine::new();
    let global_state = &GlobalState::new(SoundConfig::default());

    // add a new node
    let first_node_index = graph
        .add_node(
            NodeVariant::TestNode(TestNode {}),
            NodeInitState {
                props: &HashMap::new(),
                registry: &mut registry,
                script_engine: &scripting_engine,
                global_state,
            },
        )
        .unwrap()
        .value;

    // check that the node exists
    assert!(graph.get_node(&first_node_index).is_some());

    // now let's remove it
    assert_eq!(
        format!("{:?}", graph.remove_node(&first_node_index)),
        format!("{:?}", Ok::<(), NodeError>(()))
    );

    // let's try removing it twice
    assert_eq!(
        std::mem::discriminant(&graph.remove_node(&first_node_index).unwrap_err()),
        std::mem::discriminant(&NodeError::NodeDoesNotExist {
            node_index: NodeIndex {
                index: 0,
                generation: 0
            }
        })
    );

    // let's try to get it with its index
    assert!(graph.get_node(&first_node_index).is_none());

    // now add a second node
    let second_node_index = graph
        .add_node(
            NodeVariant::TestNode(TestNode {}),
            NodeInitState {
                props: &HashMap::new(),
                registry: &mut registry,
                script_engine: &scripting_engine,
                global_state,
            },
        )
        .unwrap()
        .value;

    // it should have taken the place of the first node
    assert_eq!(first_node_index.index, second_node_index.index);

    // as it took the place of the first one, let's make sure we can't try to
    // retrieve the old one and get the new one
    assert!(graph.get_node(&first_node_index).is_none());

    // let's see what happens if we try to delete node one
    assert_eq!(
        format!("{:?}", &graph.remove_node(&first_node_index).unwrap_err()),
        format!(
            "{:?}",
            &NodeError::NodeDoesNotExist {
                node_index: first_node_index.clone()
            }
        )
    );

    // second node should still exist though with the right generation
    assert!(graph.get_node(&second_node_index).is_some());

    // add another node for good measure to make sure it's growing
    graph
        .add_node(
            NodeVariant::TestNode(TestNode {}),
            NodeInitState {
                props: &HashMap::new(),
                registry: &mut registry,
                script_engine: &scripting_engine,
                global_state,
            },
        )
        .unwrap();
    assert_eq!(graph.len(), 2);

    println!("{:?}", graph.serialize_to_json());
}

#[test]
fn graph_connecting() {
    let mut graph = NodeGraph::new();
    let mut registry = SocketRegistry::new();
    let scripting_engine = Engine::new();
    let global_state = &GlobalState::new(SoundConfig::default());

    // add two new nodes
    let first_node_index = graph
        .add_node(
            NodeVariant::TestNode(TestNode {}),
            NodeInitState {
                props: &HashMap::new(),
                registry: &mut registry,
                script_engine: &scripting_engine,
                global_state,
            },
        )
        .unwrap()
        .value;
    let second_node_index = graph
        .add_node(
            NodeVariant::TestNode(TestNode {}),
            NodeInitState {
                props: &HashMap::new(),
                registry: &mut registry,
                script_engine: &scripting_engine,
                global_state,
            },
        )
        .unwrap()
        .value;
    let third_node_index = graph
        .add_node(
            NodeVariant::TestNode(TestNode {}),
            NodeInitState {
                props: &HashMap::new(),
                registry: &mut registry,
                script_engine: &scripting_engine,
                global_state,
            },
        )
        .unwrap()
        .value;

    // try connecting the first node to the second node with a socket
    // the the first one doesn't have
    let from_node = graph.get_node(&first_node_index).unwrap();

    assert_eq!(
        from_node.has_output_socket(&SocketType::Midi(MidiSocketType::Default)),
        false
    );

    assert_eq!(
        format!(
            "{:?}",
            graph
                .connect(
                    &first_node_index,
                    &SocketType::Midi(MidiSocketType::Default),
                    &second_node_index,
                    &SocketType::Midi(MidiSocketType::Default),
                )
                .unwrap_err()
        ),
        format!(
            "{:?}",
            NodeError::SocketDoesNotExist {
                socket_type: SocketType::Midi(MidiSocketType::Default)
            }
        )
    );

    // ditto with on the to side
    let to_node = graph.get_node(&first_node_index).unwrap();

    assert_eq!(
        to_node.has_input_socket(&SocketType::Stream(StreamSocketType::Dynamic(2))),
        false
    );

    assert_eq!(
        format!(
            "{:?}",
            graph
                .connect(
                    &first_node_index,
                    &SocketType::Stream(StreamSocketType::Audio),
                    &second_node_index,
                    &SocketType::Stream(StreamSocketType::Dynamic(2)),
                )
                .unwrap_err()
        ),
        format!(
            "{:?}",
            NodeError::SocketDoesNotExist {
                socket_type: SocketType::Stream(StreamSocketType::Dynamic(2))
            }
        )
    );

    // make sure we can't connect two different families of types (midi can't connect to audio, etc)
    assert_eq!(
        format!(
            "{:?}",
            graph
                .connect(
                    &first_node_index,
                    &SocketType::Stream(StreamSocketType::Audio),
                    &second_node_index,
                    &SocketType::Midi(MidiSocketType::Default),
                )
                .unwrap_err()
        ),
        format!(
            "{:?}",
            NodeError::IncompatibleSocketTypes {
                from: SocketType::Stream(StreamSocketType::Audio),
                to: SocketType::Midi(MidiSocketType::Default)
            }
        )
    );

    // but we should be able to connect within the same family
    assert_eq!(
        graph
            .connect(
                &first_node_index,
                &SocketType::Stream(StreamSocketType::Audio),
                &second_node_index,
                &SocketType::Stream(StreamSocketType::Audio),
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
                    &first_node_index,
                    &SocketType::Stream(StreamSocketType::Audio),
                    &second_node_index,
                    &SocketType::Stream(StreamSocketType::Audio),
                )
                .unwrap_err()
        ),
        format!(
            "{:?}",
            NodeError::AlreadyConnected {
                from: SocketType::Stream(StreamSocketType::Audio),
                to: SocketType::Stream(StreamSocketType::Audio)
            }
        )
    );

    // nor can we connect multiple outputs to one input
    assert_eq!(
        format!(
            "{:?}",
            graph
                .connect(
                    &third_node_index,
                    &SocketType::Stream(StreamSocketType::Audio),
                    &second_node_index,
                    &SocketType::Stream(StreamSocketType::Audio),
                )
                .unwrap_err()
        ),
        format!(
            "{:?}",
            NodeError::InputSocketOccupied {
                socket_type: SocketType::Stream(StreamSocketType::Audio)
            }
        )
    );

    // but we can connect one output to multiple inputs
    assert_eq!(
        graph
            .connect(
                &third_node_index,
                &SocketType::Stream(StreamSocketType::Audio),
                &second_node_index,
                &SocketType::Stream(StreamSocketType::Detune),
            )
            .is_ok(),
        true
    );
}

/// This test makes sure that when removing a node, it also removes any
/// connections from all the nodes it's connected to
#[test]
fn hanging_connections() -> Result<(), NodeError> {
    let mut graph = NodeGraph::new();
    let mut registry = SocketRegistry::new();
    let scripting_engine = Engine::new();
    let global_state = &GlobalState::new(SoundConfig::default());

    // set up a simple network
    let first_node = graph
        .add_node(
            NodeVariant::TestNode(TestNode {}),
            NodeInitState {
                props: &HashMap::new(),
                registry: &mut registry,
                script_engine: &scripting_engine,
                global_state,
            },
        )
        .unwrap()
        .value;
    let second_node = graph
        .add_node(
            NodeVariant::TestNode(TestNode {}),
            NodeInitState {
                props: &HashMap::new(),
                registry: &mut registry,
                script_engine: &scripting_engine,
                global_state,
            },
        )
        .unwrap()
        .value;

    graph.connect(
        &first_node,
        &SocketType::Stream(StreamSocketType::Audio),
        &second_node,
        &SocketType::Stream(StreamSocketType::Audio),
    )?;

    let first_node_wrapper = graph.get_node(&first_node).unwrap();
    assert_eq!(first_node_wrapper.list_connected_output_sockets().len(), 1); // it should be connected here

    graph.remove_node(&second_node)?;

    let first_node_wrapper = graph.get_node(&first_node).unwrap();
    assert_eq!(first_node_wrapper.list_connected_output_sockets().len(), 0); // it shouldn't be connected to anything

    Ok(())
}
