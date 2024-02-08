use std::fmt::Debug;

use common::resource_manager::ResourceId;

use crate::SoundConfig;

pub mod double_buffer;
pub mod envelope;
pub mod phase_calculator;
pub mod pipe_player;
pub mod rank;
pub mod rank_player;
pub mod savitzky_golay;
pub mod util;

pub trait Resource: Debug {
    fn resource_id(&self) -> &ResourceId;
}

pub trait Voice: Default {
    type Sample;
    type Resource: Resource;
    type Param: Default + Debug;

    fn new(resource: &Self::Resource, sample: &Self::Sample, sound_config: SoundConfig) -> Self;

    fn set_param(&mut self, param: &Self::Param);

    fn attack(&mut self, resource: &Self::Resource, sample: &Self::Sample);

    fn release(&mut self, resource: &Self::Resource, sample: &Self::Sample);

    fn step(&mut self, resource: &Self::Resource, sample: &Self::Sample) -> f32;

    fn reset(&mut self);

    fn active(&self) -> bool;
}
