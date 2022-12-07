use resource_manager::{ResourceId, ResourceIndex};
use smallvec::SmallVec;
use sound_engine::{midi::messages::MidiData, sampling::rank_player::RankPlayer, SoundConfig};

use crate::{
    connection::{MidiBundle, MidiSocketType, StreamSocketType},
    errors::{NodeError, NodeOk},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
    property::{Property, PropertyType},
};

#[derive(Debug, Clone)]
pub struct RankPlayerNode {
    player: Option<RankPlayer>,
    index: ResourceIndex,
    config: SoundConfig,
    polyphony: usize,
    midi_in: MidiBundle,
    out: f32,
}

impl RankPlayerNode {
    pub fn new(sound_config: &SoundConfig) -> Self {
        RankPlayerNode {
            player: None,
            index: ResourceIndex {
                index: 0,
                generation: 0,
            },
            config: sound_config.clone(),
            polyphony: 16,
            midi_in: SmallVec::new(),
            out: 0.0,
        }
    }
}

impl Node for RankPlayerNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
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

        if let Some(Some(resource)) = state.props.get("rank").map(|rank| rank.clone().as_resource()) {
            let new_index = state
                .global_state
                .resources
                .ranks
                .get_index(&resource.resource)
                .ok_or(NodeError::MissingResource { resource })?;

            did_settings_change |= new_index != self.index;
            self.index = new_index;
        } else {
            did_settings_change = false;
        }

        if self.player.is_none() || did_settings_change {
            let rank = state.global_state.resources.ranks.borrow_resource(self.index);

            if let Some(rank) = rank {
                let player = RankPlayer::new(
                    &self.config,
                    &state.global_state.resources.samples,
                    &rank,
                    self.polyphony,
                );
                self.player = Some(player);
            }
        }

        InitResult::simple(vec![
            NodeRow::Property(
                "rank".into(),
                PropertyType::Resource("ranks".into()),
                Property::Resource(ResourceId {
                    namespace: "ranks".into(),
                    resource: "".into(),
                }),
            ),
            NodeRow::Property("polyphony".into(), PropertyType::Integer, Property::Integer(16)),
            NodeRow::MidiInput(MidiSocketType::Default, SmallVec::new(), false),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0, false),
        ])
    }

    fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        if let Some(player) = &mut self.player {
            let samples = &state.global_state.resources.samples;

            for midi in &self.midi_in {
                match midi {
                    MidiData::NoteOn { note, .. } => {
                        player.play_note(*note, samples);
                    }
                    MidiData::NoteOff { note, .. } => {
                        player.release_note(*note, samples);
                    }
                    _ => {}
                }
            }

            self.midi_in.clear();

            self.out = player.next_sample(samples);
        }

        NodeOk::no_warnings(())
    }

    fn accept_midi_input(&mut self, _socket_type: &MidiSocketType, value: MidiBundle) {
        self.midi_in = value;
    }

    fn get_stream_output(&self, _socket_type: &StreamSocketType) -> f32 {
        self.out
    }
}
