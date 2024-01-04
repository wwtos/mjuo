use sound_engine::{
    node::ramp::{Ramp, RampType},
    SoundConfig,
};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct PortamentoNode {
    engaged: bool,
    active: bool,
    speed: f32,
    ramp: Ramp,
}

impl NodeRuntime for PortamentoNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        if let Some(ramp_type) = params.props.get("ramp_type") {
            let ramp_type = ramp_type.clone().as_multiple_choice().unwrap();

            match ramp_type.as_str() {
                "exponential" => self.ramp.set_ramp_type(RampType::Exponential).unwrap(),
                "linear" => self.ramp.set_ramp_type(RampType::Linear).unwrap(),
                _ => {}
            };
        }

        InitResult::nothing()
    }

    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        _midi_store: &mut MidiStoreInterface,
        _resources: &[Resource],
    ) -> NodeResult<()> {
        if let Some(gate) = ins.value(0)[0].as_boolean() {
            if self.engaged && !gate {
                outs.value(0)[0] = float(self.ramp.get_to());
                self.ramp.set_position(self.ramp.get_to());
            }

            self.engaged = gate;
            self.active = true;
        }

        if let Some(frequency) = ins.value(1)[0].as_float() {
            if self.engaged {
                self.ramp
                    .set_ramp_parameters(self.ramp.get_position(), frequency, self.speed)
                    .unwrap();
            } else {
                outs.value(0)[0] = float(frequency);
            }

            self.active = true;
        }

        if let Some(speed) = ins.value(2)[0].as_float() {
            self.speed = speed;
            self.ramp
                .set_ramp_parameters(self.ramp.get_position(), self.ramp.get_to(), self.speed)
                .unwrap();

            self.active = true;
        }

        if self.engaged && self.active {
            let out = self.ramp.process();

            outs.value(0)[0] = float(out);

            if self.ramp.is_done() {
                self.active = false;
            }
        } else if self.active {
            self.active = false;
        }

        NodeOk::no_warnings(())
    }
}

impl Node for PortamentoNode {
    fn new(sound_config: &SoundConfig) -> PortamentoNode {
        PortamentoNode {
            engaged: true,
            active: true,
            speed: 0.2,
            ramp: Ramp::new_with_start_value(sound_config.sample_rate as f32 / sound_config.buffer_size as f32, 440.0),
        }
    }

    fn get_io(_context: &NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            multiple_choice("ramp_type", &["exponential", "linear"], "exponential"),
            value_input("gate", Primitive::Boolean(false), 1),
            value_input("frequency", Primitive::Float(440.0), 1),
            value_input("speed", Primitive::Float(0.2), 1),
            value_output("frequency", 1),
        ])
    }
}
