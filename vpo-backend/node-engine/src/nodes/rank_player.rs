use sound_engine::{
    sampling::{
        percussion_player::{PercussionParam, PercussionPlayer},
        pipe_player::{PipeParam, PipePlayer},
        rank::{Pipe, RankType},
        rank_player::RankPlayer,
    },
    util::{cents_to_detune, db_to_gain},
};

use crate::nodes::prelude::*;

#[derive(Debug)]
enum PlayerType {
    Pipe(RankPlayer<PipePlayer>, PipeParam),
    Percussion(RankPlayer<PercussionPlayer>, PercussionParam),
}

#[derive(Debug)]
pub struct RankPlayerNode {
    player: Option<PlayerType>,
    polyphony: usize,
}

impl NodeRuntime for RankPlayerNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let NodeInitParams { props, resources, .. } = params;

        self.player = None;

        self.polyphony = props.get_int("polyphony")?.clamp(1, 255) as usize;

        let rank_id = props.get_resource("rank")?;
        let rank_type = props.get_multiple_choice("rank_type")?;

        let rank = resources.ranks.borrow_resource_by_id(&rank_id.resource);

        let player_and_resources = rank.and_then(|rank| match rank {
            RankType::Pipes(pipe_rank) => {
                // TODO: figure if there's a more elegant way to do this
                if rank_type != "pipe" {
                    return None;
                }

                let (player, needed_resources) =
                    RankPlayer::new(rank_id.clone(), pipe_rank, self.polyphony, params.sound_config.clone());

                Some((PlayerType::Pipe(player, PipeParam::default()), needed_resources))
            }
            RankType::Percussion(percussion_rank) => {
                if rank_type != "percussion" {
                    return None;
                }

                let (player, needed_resources) = RankPlayer::new(
                    rank_id.clone(),
                    percussion_rank,
                    self.polyphony,
                    params.sound_config.clone(),
                );

                Some((
                    PlayerType::Percussion(player, PercussionParam::default()),
                    needed_resources,
                ))
            }
        });

        if let Some((player, needed_resources)) = player_and_resources {
            self.player = Some(player);

            NodeOk::no_warnings(InitResult {
                changed_properties: None,
                needed_resources,
            })
        } else {
            Ok(NodeOk {
                value: InitResult::default(),
                warnings: vec![NodeWarning::ResourceMissing { resource: rank_id }],
            })
        }
    }

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut OscStore,
        resources: &[Resource],
    ) {
        let midi_in = ins.midi(0)[0]
            .as_ref()
            .and_then(|x| midi_store.borrow_osc(x))
            .unwrap_or(&[]);

        for frame in outs.stream(0)[0].iter_mut() {
            *frame = 0.0;
        }

        match &mut self.player {
            Some(PlayerType::Pipe(player, param)) => {
                let mut dirty = false;

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
                    player.set_param(param.clone());
                }

                if let Some(Resource::Rank(RankType::Pipes(rank))) = resources.get(0) {
                    player.next_buffered(
                        context.current_time,
                        midi_in,
                        rank,
                        &resources[1..],
                        &mut outs.stream(0)[0],
                    );
                }
            }
            Some(PlayerType::Percussion(player, param)) => {
                if let Some(Resource::Rank(RankType::Percussion(rank))) = resources.get(0) {
                    let mut dirty = false;

                    if let Some(cents) = ins.value(0)[0].as_float() {
                        param.detune = cents_to_detune(cents);
                        dirty = true;
                    }

                    if let Some(db_gain) = ins.value(1)[0].as_float() {
                        param.gain = db_to_gain(db_gain);
                        dirty = true;
                    }

                    if dirty {
                        player.set_param(param.clone());
                    }

                    player.next_buffered(
                        context.current_time,
                        midi_in,
                        rank,
                        &resources[1..],
                        &mut outs.stream(0)[0],
                    );
                }
            }
            None => {}
        }
    }
}

impl Node for RankPlayerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        RankPlayerNode {
            player: None,
            polyphony: 16,
        }
    }

    fn get_io(_context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
        let mut rows = vec![
            multiple_choice("rank_type", &["pipe", "percussion"], "pipe"),
            resource("rank", "ranks"),
            property("polyphony", PropertyType::Integer, Property::Integer(16)),
            midi_input("midi", 1),
            value_input("detune", Primitive::Float(0.0), 1),
            value_input("db_gain", Primitive::Float(0.0), 1),
        ];

        match props
            .get_multiple_choice("rank_type")
            .unwrap_or("pipe".to_string())
            .as_str()
        {
            "percussion" => {}
            _ => rows.push(value_input("shelf_db_gain", Primitive::Float(0.0), 1)),
        };

        rows.push(stream_output("audio", 1));

        NodeIo::simple(rows)
    }
}
