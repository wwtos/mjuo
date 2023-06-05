use resource_manager::{ResourceId, ResourceIndex};
use smallvec::SmallVec;
use sound_engine::{
    sampling::rank_player::RankPlayer,
    util::{cents_to_detune, db_to_gain},
};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct RankPlayerNode {
    player: Option<RankPlayer>,
    index: Option<ResourceIndex>,
    rank_resource: Option<ResourceId>,
    polyphony: usize,
    midi_in: MidiBundle,
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
            let new_index =
                state
                    .resources
                    .ranks
                    .get_index(&resource.resource)
                    .ok_or_else(|| NodeError::MissingResource {
                        resource: resource.clone(),
                    })?;

            did_settings_change |= Some(new_index) != self.index;
            self.index = Some(new_index);
            self.rank_resource = Some(resource);
        } else {
            did_settings_change = false;
        }

        if self.player.is_none() || did_settings_change {
            let rank = state.resources.ranks.borrow_resource(self.index.unwrap());

            if let Some(rank) = rank {
                let player = RankPlayer::new(
                    &state.resources.samples,
                    rank,
                    self.polyphony,
                    state.sound_config.sample_rate,
                );
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
        let mut reset_needed = false;

        let rank = if let Some(index) = self.index {
            if let Some(rank) = state.resources.ranks.borrow_resource(index) {
                Some(rank)
            } else {
                // index must have changed
                let new_index = state
                    .resources
                    .ranks
                    .get_index(&self.rank_resource.as_ref().unwrap().resource);
                self.index = new_index;

                reset_needed = true;

                new_index.and_then(|new_index| state.resources.ranks.borrow_resource(new_index))
            }
        } else {
            None
        };

        if let (Some(player), Some(rank)) = (&mut self.player, rank) {
            let samples = &state.resources.samples;

            if reset_needed {
                player.reset();
            }

            for frame in streams_out[0].iter_mut() {
                *frame = 0.0;
            }

            player.next_buffered(rank, state.current_time, &self.midi_in, samples, streams_out[0]);
        }

        NodeOk::no_warnings(())
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let Some(player) = &mut self.player {
            if let Some(cents) = values_in[0].as_ref().and_then(|x| x.as_float()) {
                player.set_detune(cents_to_detune(cents));
            }

            if let Some(db_gain) = values_in[1].as_ref().and_then(|x| x.as_float()) {
                player.set_gain(db_to_gain(db_gain));
            }

            if let Some(shelf_db_gain) = values_in[2].as_ref().and_then(|x| x.as_float()) {
                player.set_shelf_db_gain(shelf_db_gain);
            }
        }
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        let value = midi_in[0].clone().unwrap();

        self.midi_in = value;
    }
}

impl Node for RankPlayerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        RankPlayerNode {
            player: None,
            index: None,
            rank_resource: None,
            polyphony: 16,
            midi_in: SmallVec::new(),
        }
    }

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
            value_input(register("detune"), Primitive::Float(0.0)),
            value_input(register("db_gain"), Primitive::Float(0.0)),
            value_input(register("shelf_db_gain"), Primitive::Float(0.0)),
            stream_output(register("audio")),
        ])
    }
}
