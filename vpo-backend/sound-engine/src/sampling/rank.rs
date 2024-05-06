use std::collections::BTreeMap;
use std::fmt::Debug;

use common::resource_manager::ResourceId;

use super::{phase_calculator::PhaseCalculator, pipe_player::EnvelopeIndexes, Resource};

#[derive(Debug)]
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

    pub phase_calculator: PhaseCalculator,
    pub amp_window_size: usize,
    pub attack_envelope: EnvelopeIndexes,
    pub release_envelope: EnvelopeIndexes,
}

impl Resource for Pipe {
    fn resource_id(&self) -> &ResourceId {
        &self.resource
    }
}

#[derive(Debug)]
pub struct Percussion {
    pub resource: ResourceId,
    /// in seconds
    pub release_duration: f32,
    pub gain: f32,
}

impl Resource for Percussion {
    fn resource_id(&self) -> &ResourceId {
        &self.resource
    }
}

#[derive(Debug)]
pub struct Rank<T: Debug> {
    pub notes: BTreeMap<u8, T>,
    pub name: String,
}

#[derive(Debug)]
pub enum RankType {
    Pipes(Rank<Pipe>),
    Percussion(Rank<Percussion>),
}

impl RankType {
    pub fn as_pipes(&self) -> Option<&Rank<Pipe>> {
        match self {
            RankType::Pipes(pipes) => Some(pipes),
            _ => None,
        }
    }
}
