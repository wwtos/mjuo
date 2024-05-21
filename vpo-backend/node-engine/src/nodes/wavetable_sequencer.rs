use common::resource_manager::ResourceId;
use common::traits::TryRef;
use sound_engine::{util::interpolate::lerp, SoundConfig};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct WavetableSequencerNode {
    /// 0-1, not 0-TWO_PI
    phase: f32,
    frequency: f32,
    advance_by: f32,
}

impl NodeRuntime for WavetableSequencerNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let needed_resource = params.props.get_resource("wavetable")?;

        NodeOk::no_warnings(InitResult {
            changed_properties: None,
            needed_resources: vec![needed_resource],
        })
    }

    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        _midi_store: &mut OscStore,
        resources: &[Resource],
    ) {
        if let Some(frequency) = ins.value(0)[0].as_float() {
            self.frequency = frequency;
        }

        if let Ok(sample) = resources[0].try_ref() {
            let wavetable = &sample.audio_raw;

            let wavetable_pos = self.phase * wavetable.len() as f32;

            let wavetable_index = wavetable_pos as usize;
            let wavetable_offset = wavetable_pos.fract();

            outs.value(0)[0] = float(lerp(
                wavetable[wavetable_index],
                wavetable[(wavetable_index + 1) % wavetable.len()],
                wavetable_offset,
            ));

            self.phase += self.advance_by * self.frequency;
            self.phase = self.phase.fract();
        }
    }
}

impl Node for WavetableSequencerNode {
    fn new(config: &SoundConfig) -> Self {
        let advance_by = (config.buffer_size as f32) / (config.sample_rate as f32);

        WavetableSequencerNode {
            phase: 0.0,
            frequency: 1.0,
            advance_by,
        }
    }

    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            property(
                "wavetable",
                PropertyType::Resource("samples".into()),
                Property::Resource(ResourceId {
                    namespace: "samples".into(),
                    resource: "".into(),
                }),
            ),
            value_input("frequency", Primitive::Float(2.0), 1),
            value_output("value", 1),
        ])
    }
}
