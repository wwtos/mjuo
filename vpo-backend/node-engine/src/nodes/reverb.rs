use std::fmt::Debug;

use serde::{ser::SerializeMap, Serializer};
use sound_engine::node::zita_rev1::{self, ZitaRev1};

use crate::nodes::prelude::*;

#[derive(Clone)]
pub struct ReverbNode {
    zita1: Box<ZitaRev1>,
}

impl Debug for ReverbNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.serialize_struct("ReverbNode", 0)?.end()
    }
}

impl NodeRuntime for ReverbNode {
    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        _midi_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        for i in 0..11 {
            let param_in = ins.value(i)[0];

            if let Some(param_in) = param_in.as_float() {
                let param_index = zita_rev1::ParamIndex::from_repr(i as u8).unwrap();

                self.zita1.set_param(param_index, param_in);
            }
        }

        let mut audio_out = outs.stream(0);
        let mut iter = audio_out.iter_mut();

        let output0 = iter.next().unwrap();
        let output1 = iter.next().unwrap();

        self.zita1
            .compute(2, &ins.stream(0)[0], &ins.stream(0)[1], output0, output1);
    }
}

impl Node for ReverbNode {
    fn new(sound_config: &SoundConfig) -> Self {
        let mut reverb = ZitaRev1::new();
        reverb.init(sound_config.sample_rate as i32);

        ReverbNode {
            zita1: Box::new(reverb),
        }
    }

    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            stream_input("audio", 2),
            stream_output("audio", 2),
            value_input("zita_delay", Primitive::Float(60.0), 1),
            value_input("zita_crossover", Primitive::Float(200.0), 1),
            value_input("zita_low_rt60", Primitive::Float(3.0), 1),
            value_input("zita_mid_rt60", Primitive::Float(2.0), 1),
            value_input("zita_hf_damping", Primitive::Float(6000.0), 1),
            value_input("zita_eq1_freq", Primitive::Float(315.0), 1),
            value_input("zita_eq1_level", Primitive::Float(0.0), 1),
            value_input("zita_eq2_freq", Primitive::Float(1500.0), 1),
            value_input("zita_eq2_level", Primitive::Float(0.0), 1),
            value_input("zita_dry_wet_mix", Primitive::Float(0.0), 1),
            value_input("zita_level", Primitive::Float(0.0), 1),
        ])
    }
}
