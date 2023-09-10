use lazy_static::lazy_static;
use resource_manager::ResourceId;
use smallvec::SmallVec;
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
    pub static ref EMPTY_MIDI: MidiBundle = SmallVec::new();
}

impl NodeRuntime for RankPlayerNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let mut did_settings_change = false;

        if let Some(polyphony) = params
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

    fn process(
        &mut self,
        context: NodeProcessContext,
        ins: Ins,
        outs: Outs,
        resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        let _reset_needed = false;

        if resources[0].is_some() {
            if let Some(cents) = ins.values[0].as_ref().and_then(|x| x.as_float()) {
                self.player.set_detune(cents_to_detune(cents));
            }

            if let Some(db_gain) = ins.values[1].as_ref().and_then(|x| x.as_float()) {
                self.player.set_gain(db_to_gain(db_gain));
            }

            if let Some(shelf_db_gain) = ins.values[2].as_ref().and_then(|x| x.as_float()) {
                self.player.set_shelf_db_gain(shelf_db_gain);
            }

            for frame in outs.streams[0].iter_mut() {
                *frame = 0.0;
            }

            self.player.next_buffered(
                context.current_time,
                ins.midis[0].as_ref().unwrap_or(&EMPTY_MIDI),
                resources,
                outs.streams[0],
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

    fn get_io(_props: HashMap<String, Property>) -> NodeIo {
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
            midi_input("midi"),
            value_input("detune", Primitive::Float(0.0)),
            value_input("db_gain", Primitive::Float(0.0)),
            value_input("shelf_db_gain", Primitive::Float(0.0)),
            stream_output("audio"),
        ])
    }
}
