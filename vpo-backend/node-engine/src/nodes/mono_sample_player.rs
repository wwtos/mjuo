use resource_manager::{ResourceId, ResourceIndex};
use sound_engine::sampling::sample_player::SamplePlayer;

use crate::{
    connection::{Primitive, StreamSocketType, ValueSocketType},
    errors::{NodeError, NodeOk},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
    property::{Property, PropertyType},
};

#[derive(Debug, Clone)]
pub struct MonoSamplePlayerNode {
    player: Option<SamplePlayer>,
    released: bool,
    played: bool,
    index: ResourceIndex,
    output: f32,
}

impl Default for MonoSamplePlayerNode {
    fn default() -> Self {
        MonoSamplePlayerNode {
            player: None,
            released: false,
            played: false,
            index: ResourceIndex {
                index: 0,
                generation: 0,
            },
            output: 0.0,
        }
    }
}

impl Node for MonoSamplePlayerNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        let did_index_change;

        if let Some(Some(resource)) = state.props.get("sample").map(|sample| sample.clone().as_resource()) {
            let new_index = state
                .global_state
                .resources
                .samples
                .get_index(&resource.resource)
                .ok_or(NodeError::MissingResource { resource })?;

            did_index_change = new_index != self.index;
            self.index = new_index;
        } else {
            did_index_change = false;
        }

        if self.player.is_none() || did_index_change {
            let sample = state.global_state.resources.samples.borrow_resource(self.index);

            if let Some(sample) = sample {
                self.player = Some(SamplePlayer::new(&sample));
            }
        }

        InitResult::simple(vec![
            NodeRow::Property(
                "sample".into(),
                PropertyType::Resource("samples".into()),
                Property::Resource(ResourceId {
                    namespace: "samples".into(),
                    resource: "".into(),
                }),
            ),
            NodeRow::ValueInput(ValueSocketType::Default, Primitive::Boolean(false), false),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0, false),
        ])
    }

    fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        if let Some(player) = &mut self.player {
            let sample = state
                .global_state
                .resources
                .samples
                .borrow_resource(self.index)
                .unwrap();

            if self.played {
                player.play(sample);
                self.played = false;
            }

            if self.released {
                player.release(sample);
                self.released = false;
            }

            self.output = player.next_sample(&sample);
        }

        NodeOk::no_warnings(())
    }

    fn accept_value_input(&mut self, _socket_type: ValueSocketType, value: Primitive) {
        if let Some(player) = &mut self.player {
            if let Some(engaged) = value.as_boolean() {
                if engaged {
                    self.played = true;
                } else {
                    self.released = true;
                }
            }
        }
    }

    fn get_stream_output(&self, _socket_type: StreamSocketType) -> f32 {
        self.output
    }
}
