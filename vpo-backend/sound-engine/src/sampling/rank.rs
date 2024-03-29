use std::collections::BTreeMap;

use common::resource_manager::ResourceId;
use serde::Serialize;

use super::{phase_calculator::PhaseCalculator, pipe_player::EnvelopeIndexes};

#[derive(Debug, Serialize)]
pub struct Pipe {
    pub resource: ResourceId,

    /// freq can be quite different from the note in the case of non 8' pipes
    /// (e.g. 4' or 2 2/3')
    pub freq: f32,

    pub amplitude: f32,
    pub comb_coeff: f32,

    pub loop_start: usize,
    pub loop_end: usize,
    pub decay_index: usize,
    pub release_index: usize,

    pub crossfade: usize,

    #[serde(skip)]
    pub phase_calculator: PhaseCalculator,
    #[serde(skip)]
    pub amp_window_size: usize,
    #[serde(skip)]
    pub attack_envelope: EnvelopeIndexes,
    #[serde(skip)]
    pub release_envelope: EnvelopeIndexes,
}

#[derive(Debug, Serialize)]
pub struct Rank {
    pub pipes: BTreeMap<u8, Pipe>,
    pub name: String,
}
