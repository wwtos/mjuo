use common::traits::TryRef;
use sound_engine::{node::wavetable_oscillator::WavetableOscillator, SoundConfig};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct WavetableNode {
    oscillator: WavetableOscillator,
}

impl NodeRuntime for WavetableNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let needed_resource = params.props.get("wavetable").and_then(|x| x.clone().as_resource());

        self.oscillator = WavetableOscillator::new(params.sound_config.clone());

        NodeOk::no_warnings(InitResult {
            changed_properties: None,
            needed_resources: needed_resource.map(|x| vec![x]).unwrap_or(vec![]),
        })
    }

    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        _midi_store: &mut MidiStore,
        resources: &[Resource],
    ) {
        if let Some(frequency) = ins.value(0)[0].as_float() {
            self.oscillator.set_frequency(frequency);
        }

        if let Some(wavetable) = resources.get(0).and_then(|x| x.try_ref().ok()) {
            for frame in outs.stream(0)[0].iter_mut() {
                *frame = self.oscillator.get_next_sample(wavetable);
            }
        }
    }
}

impl Node for WavetableNode {
    fn new(config: &SoundConfig) -> Self {
        WavetableNode {
            oscillator: WavetableOscillator::new(config.clone()),
        }
    }

    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            resource("wavetable", "samples"),
            value_input("frequency", Primitive::Float(440.0), 1),
            stream_output("audio", 1),
        ])
    }
}
