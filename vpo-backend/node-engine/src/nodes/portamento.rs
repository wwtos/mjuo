use sound_engine::{
    node::ramp::{Ramp, RampType},
    SoundConfig,
};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct PortamentoNode {
    value_out: Option<Primitive>,
    engaged: bool,
    active: bool,
    speed: f32,
    ramp: Ramp,
}

impl NodeRuntime for PortamentoNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        if let Some(ramp_type) = state.props.get("ramp_type") {
            let ramp_type = ramp_type.clone().as_multiple_choice().unwrap();

            match ramp_type.as_str() {
                "exponential" => self.ramp.set_ramp_type(RampType::Exponential).unwrap(),
                "linear" => self.ramp.set_ramp_type(RampType::Linear).unwrap(),
                _ => {}
            };
        }

        InitResult::nothing()
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        if self.engaged && self.active {
            let out = self.ramp.process();

            self.value_out = Some(Primitive::Float(out));

            if self.ramp.is_done() {
                self.active = false;
                self.value_out = None;
            }
        } else if self.active {
            self.active = false;
        } else if !self.active && self.value_out.is_some() {
            self.value_out = None;
        }

        NodeOk::no_warnings(())
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let [gate, frequency, speed] = values_in {
            if let Some(gate) = gate.clone().and_then(|x| x.as_boolean()) {
                if self.engaged && !gate {
                    self.value_out = Some(Primitive::Float(self.ramp.get_to()));
                    self.ramp.set_position(self.ramp.get_to());
                }

                self.engaged = gate;
            }

            if let Some(frequency) = frequency.clone().and_then(|x| x.as_float()) {
                if self.engaged {
                    self.ramp
                        .set_ramp_parameters(self.ramp.get_position(), frequency, self.speed)
                        .unwrap();
                } else {
                    self.value_out = Some(Primitive::Float(frequency));
                }
            }

            if let Some(speed) = speed.clone().and_then(|x| x.as_float()) {
                self.speed = speed;
                self.ramp
                    .set_ramp_parameters(self.ramp.get_position(), self.ramp.get_to(), self.speed)
                    .unwrap();
            }
        }

        self.active = true;
    }

    fn get_value_outputs(&self, values_out: &mut [Option<Primitive>]) {
        values_out[0] = self.value_out.clone();
    }
}

impl Node for PortamentoNode {
    fn new(sound_config: &SoundConfig) -> PortamentoNode {
        PortamentoNode {
            value_out: None,
            engaged: true,
            active: true,
            speed: 0.2,
            ramp: Ramp::new_with_start_value(sound_config, 440.0),
        }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            NodeRow::Property(
                "ramp_type".into(),
                PropertyType::MultipleChoice(vec!["exponential".into(), "linear".into()]),
                Property::MultipleChoice("exponential".into()),
            ),
            value_input(register("gate"), Primitive::Boolean(false)),
            value_input(register("frequency"), Primitive::Float(440.0)),
            value_input(register("speed"), Primitive::Float(0.2)),
            value_output(register("frequency"), Primitive::Float(440.0)),
        ])
    }
}
