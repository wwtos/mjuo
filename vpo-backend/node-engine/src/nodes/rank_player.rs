use resource_manager::{ResourceId, ResourceIndex};
use smallvec::SmallVec;
use sound_engine::{midi::messages::MidiData, sampling::rank_player::RankPlayer};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct RankPlayerNode {
    player: Option<RankPlayer>,
    index: Option<ResourceIndex>,
    polyphony: usize,
    midi_in: MidiBundle,
}

impl Default for RankPlayerNode {
    fn default() -> Self {
        RankPlayerNode {
            player: None,
            index: None,
            polyphony: 16,
            midi_in: SmallVec::new(),
        }
    }
}

impl NodeRuntime for RankPlayerNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let mut did_settings_change = false;

        if let Some(polyphony) = state
            .props
            .get("polyphony")
            .and_then(|polyphony| polyphony.clone().as_integer())
        {
            let polyphony = polyphony.max(1);

            if polyphony != self.polyphony as i32 {
                did_settings_change |= true;
            }

            self.polyphony = polyphony as usize;
        }

        if let Some(resource) = state.props.get("rank").and_then(|rank| rank.clone().as_resource()) {
            let new_index = state
                .global_state
                .resources
                .ranks
                .get_index(&resource.resource)
                .ok_or(NodeError::MissingResource { resource })?;

            did_settings_change |= Some(new_index) != self.index;
            self.index = Some(new_index);
        } else {
            did_settings_change = false;
        }

        if self.player.is_none() || did_settings_change {
            let rank = state.global_state.resources.ranks.borrow_resource(self.index.unwrap());

            if let Some(rank) = rank {
                let player = RankPlayer::new(&state.global_state.resources.samples, &rank, self.polyphony);
                self.player = Some(player);
            }
        }

        InitResult::nothing()
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        _streams_in: &[&[f32]],
        streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        let rank = state.global_state.resources.ranks.borrow_resource(self.index.unwrap());

        if let (Some(player), Some(rank)) = (&mut self.player, rank) {
            let samples = &state.global_state.resources.samples;

            if !self.midi_in.is_empty() {
                for midi in &self.midi_in {
                    match midi {
                        MidiData::NoteOn { note, .. } => {
                            player.play_note(*note, rank, samples);
                        }
                        MidiData::NoteOff { note, .. } => {
                            player.release_note(*note, rank, samples);
                        }
                        _ => {}
                    }
                }

                self.midi_in.clear();
            }

            for frame in streams_out[0].iter_mut() {
                *frame = 0.0;
            }

            player.next_buffered(rank, samples, streams_out[0]);
        }

        NodeOk::no_warnings(())
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        let value = midi_in[0].clone().unwrap();

        self.midi_in = value;
    }
}

impl Node for RankPlayerNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            NodeRow::Property(
                "rank".into(),
                PropertyType::Resource("ranks".into()),
                Property::Resource(ResourceId {
                    namespace: "ranks".into(),
                    resource: "".into(),
                }),
            ),
            NodeRow::Property("polyphony".into(), PropertyType::Integer, Property::Integer(16)),
            midi_input(register("midi"), SmallVec::new()),
            stream_output(register("audio"), 0.0),
        ])
    }
}
