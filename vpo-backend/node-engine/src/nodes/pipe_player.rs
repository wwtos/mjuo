use resource_manager::{ResourceId, ResourceIndex};
use sound_engine::sampling::pipe_player::PipePlayer;
use web_sys::console;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct PipePlayerNode {
    player: Option<PipePlayer>,
    released: bool,
    played: bool,
    index: Option<ResourceIndex>,
}

impl Default for PipePlayerNode {
    fn default() -> Self {
        PipePlayerNode {
            player: None,
            released: false,
            played: false,
            index: None,
        }
    }
}

impl NodeRuntime for PipePlayerNode {
    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let did_index_change;

        if let Some(Some(resource)) = state.props.get("pipe").map(|pipe| pipe.clone().as_resource()) {
            let new_index = state
                .global_state
                .resources
                .pipes
                .get_index(&resource.resource)
                .ok_or(NodeError::MissingResource { resource })?;

            did_index_change = Some(new_index) != self.index;
            self.index = Some(new_index);
        } else {
            did_index_change = false;
        }

        if self.player.is_none() || did_index_change {
            let pipe = state.global_state.resources.pipes.borrow_resource(self.index.unwrap());

            if let Some(pipe) = pipe {
                self.player = Some(PipePlayer::new(&pipe));
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
        if let (Some(player), Some(index)) = (&mut self.player, &self.index) {
            let pipe = state.global_state.resources.pipes.borrow_resource(*index).unwrap();

            if self.played {
                player.play(pipe);
                self.played = false;
            }

            if self.released {
                player.release(pipe);
                self.released = false;
            }

            streams_out[0] = player.next_sample(&pipe);
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

impl Node for PipePlayerNode {
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            NodeRow::Property(
                "pipe".into(),
                PropertyType::Resource("pipes".into()),
                Property::Resource(ResourceId {
                    namespace: "pipes".into(),
                    resource: "".into(),
                }),
            ),
            value_input(register("value"), Primitive::Boolean(false)),
            stream_output(register("audio"), 0.0),
        ])
    }
}
