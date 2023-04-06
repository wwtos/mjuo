use resource_manager::{ResourceId, ResourceIndex};
use sound_engine::sampling::sample_player::SamplePlayer;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct MonoSamplePlayerNode {
    player: Option<SamplePlayer>,
    released: bool,
    played: bool,
    index: Option<ResourceIndex>,
}

impl Default for MonoSamplePlayerNode {
    fn default() -> Self {
        MonoSamplePlayerNode {
            player: None,
            released: false,
            played: false,
            index: None,
        }
    }
}

impl NodeRuntime for MonoSamplePlayerNode {
    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let did_index_change;

        if let Some(Some(resource)) = state.props.get("sample").map(|sample| sample.clone().as_resource()) {
            let new_index = state
                .global_state
                .resources
                .samples
                .get_index(&resource.resource)
                .ok_or(NodeError::MissingResource { resource })?;

            did_index_change = Some(new_index) != self.index;
            self.index = Some(new_index);
        } else {
            did_index_change = false;
        }

        if self.player.is_none() || did_index_change {
            let sample = state
                .global_state
                .resources
                .samples
                .borrow_resource(self.index.unwrap());

            if let Some(sample) = sample {
                self.player = Some(SamplePlayer::new(&sample));
            }
        }

        InitResult::nothing()
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        _streams_in: &[f32],
        streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
        if let Some(player) = &mut self.player {
            let sample = state
                .global_state
                .resources
                .samples
                .borrow_resource(self.index.unwrap())
                .unwrap();

            if self.played {
                player.play(sample);
                self.played = false;
            }

            if self.released {
                player.release(sample);
                self.released = false;
            }

            streams_out[0] = player.next_sample(&sample);
        }

        NodeOk::no_warnings(())
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let [Some(engaged)] = values_in {
            if let Some(engaged) = engaged.clone().as_boolean() {
                if engaged {
                    self.played = true;
                } else {
                    self.released = true;
                }
            }
        }
    }
}

impl Node for MonoSamplePlayerNode {
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            NodeRow::Property(
                "sample".into(),
                PropertyType::Resource("samples".into()),
                Property::Resource(ResourceId {
                    namespace: "samples".into(),
                    resource: "".into(),
                }),
            ),
            value_input(register("value"), Primitive::Boolean(false)),
            stream_output(register("audio"), 0.0),
        ])
    }
}
