use common::resource_manager::ResourceId;
use lazy_static::lazy_static;
use sound_engine::{
    sampling::{
        pipe_player::{PipeParam, PipePlayer},
        rank_player::RankPlayer,
    },
    util::{cents_to_detune, db_to_gain},
};

use crate::nodes::prelude::*;

#[derive(Debug)]
pub struct RankPlayerNode {
    player: RankPlayer<PipePlayer>,
    polyphony: usize,
}

lazy_static! {
    pub static ref EMPTY_MIDI: MidiChannel = Vec::new();
}

impl NodeRuntime for RankPlayerNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        if let Some(polyphony) = params
            .props
            .get("polyphony")
            .and_then(|polyphony| polyphony.clone().as_integer())
        {
            let polyphony = polyphony.max(1);

            self.polyphony = polyphony as usize;
        }

        let rank_resource_id = params.props.get("rank").and_then(|x| x.clone().as_resource());

        let rank = rank_resource_id
            .as_ref()
            .and_then(|resource_id| params.resources.ranks.borrow_resource_by_id(&resource_id.resource));

        let needed_resources = if let Some(resource_id) = rank_resource_id {
            if let Some(rank) = rank {
                let (player, needed_resources) =
                    RankPlayer::new(resource_id, rank, self.polyphony, params.sound_config.clone());

                self.player = player;

                needed_resources
            } else {
                return Ok(NodeOk {
                    value: InitResult::default(),
                    warnings: vec![NodeWarning::ResourceMissing { resource: resource_id }],
                });
            }
        } else {
            vec![]
        };

        NodeOk::no_warnings(InitResult {
            changed_properties: None,
            needed_resources,
        })
    }

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut MidiStore,
        resources: &[Resource],
    ) {
        let _reset_needed = false;

        let mut dirty = false;
        let mut param = PipeParam::default();

        if let Some(cents) = ins.value(0)[0].as_float() {
            param.detune = cents_to_detune(cents);
            dirty = true;
        }

        if let Some(db_gain) = ins.value(1)[0].as_float() {
            param.gain = db_to_gain(db_gain);
            dirty = true;
        }

        if let Some(shelf_db_gain) = ins.value(2)[0].as_float() {
            param.third_db_gain = shelf_db_gain;
            dirty = true;
        }

        if dirty {
            self.player.set_param(param);
        }

        for frame in outs.stream(0)[0].iter_mut() {
            *frame = 0.0;
        }

        if let Some(Resource::Rank(rank)) = resources.get(0) {
            let midi_in = ins.midi(0)[0]
                .as_ref()
                .and_then(|x| midi_store.borrow_midi(x))
                .unwrap_or(&[]);

            self.player.next_buffered(
                context.current_time,
                midi_in,
                rank,
                &resources[1..],
                &mut outs.stream(0)[0],
            );
        }
    }
}

impl Node for RankPlayerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        RankPlayerNode {
            player: RankPlayer::default(),
            polyphony: 16,
        }
    }

    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
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
            midi_input("midi", 1),
            value_input("detune", Primitive::Float(0.0), 1),
            value_input("db_gain", Primitive::Float(0.0), 1),
            value_input("shelf_db_gain", Primitive::Float(0.0), 1),
            stream_output("audio", 1),
        ])
    }
}
