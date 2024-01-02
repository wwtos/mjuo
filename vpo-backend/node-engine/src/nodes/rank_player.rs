use common::resource_manager::ResourceId;
use lazy_static::lazy_static;
use sound_engine::{
    sampling::rank_player::RankPlayer,
    util::{cents_to_detune, db_to_gain},
};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct RankPlayerNode {
    player: RankPlayer,
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

        let rank_resource = params
            .props
            .get("rank")
            .and_then(|x| x.clone().as_resource())
            .and_then(|resource_id| {
                params
                    .resources
                    .ranks
                    .borrow_resource_by_id(&resource_id.resource)
                    .map(|resource| (resource_id.clone(), resource))
            });

        let needed_resources = if let Some((id, rank)) = rank_resource {
            let (player, needed_resources) = RankPlayer::new(id, rank, self.polyphony, params.sound_config.sample_rate);

            self.player = player;

            needed_resources
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
        midi_store: &mut MidiStoreInterface,
        resources: &[Resource],
    ) -> NodeResult<()> {
        let _reset_needed = false;

        if let Some(cents) = ins.value(0)[0].as_float() {
            self.player.set_detune(cents_to_detune(cents));
        }

        if let Some(db_gain) = ins.value(1)[0].as_float() {
            self.player.set_gain(db_to_gain(db_gain));
        }

        if let Some(shelf_db_gain) = ins.value(2)[0].as_float() {
            self.player.set_shelf_db_gain(shelf_db_gain);
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

        NodeOk::no_warnings(())
    }
}

impl Node for RankPlayerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        RankPlayerNode {
            player: RankPlayer::default(),
            polyphony: 16,
        }
    }

    fn get_io(_context: &NodeGetIoContext, _props: HashMap<String, Property, BuildHasherDefault<SeaHasher>>) -> NodeIo {
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
