use std::any::TypeId;

use resource_manager::ResourceId;
use smallvec::SmallVec;
use sound_engine::{
    sampling::{rank::Rank, rank_player::RankPlayer},
    util::{cents_to_detune, db_to_gain},
};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct RankPlayerNode {
    player: RankPlayer,
    polyphony: usize,
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

        let rank_resource = state
            .props
            .get("rank")
            .and_then(|x| x.as_resource())
            .and_then(|resource_id| {
                state
                    .resources
                    .ranks
                    .borrow_resource_by_id(&resource_id.resource)
                    .map(|resource| (resource_id.clone(), resource))
            });

        let needed_resources = if let Some((id, rank)) = rank_resource {
            let (player, needed_resources) = RankPlayer::new(id, rank, self.polyphony, state.sound_config.sample_rate);

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
        globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        resources: &[(ResourceIndex, &dyn Any)],
    ) -> NodeResult<()> {
        let mut reset_needed = false;

        if resources[0].1.type_id() == TypeId::of::<Rank>() {
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
                globals.current_time,
                &ins.midis[0].unwrap_or_else(|| SmallVec::new()),
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
            midi_input(register("midi")),
            value_input(register("detune"), Primitive::Float(0.0)),
            value_input(register("db_gain"), Primitive::Float(0.0)),
            value_input(register("shelf_db_gain"), Primitive::Float(0.0)),
            stream_output(register("audio")),
        ])
    }
}
