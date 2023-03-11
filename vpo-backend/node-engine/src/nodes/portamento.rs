use sound_engine::{
    node::ramp::{Ramp, RampType},
    SoundConfig,
};

use crate::{
    connection::{Primitive, ValueSocketType},
    errors::{NodeError, NodeOk, NodeResult},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
    property::{Property, PropertyType},
};

#[derive(Debug, Clone)]
pub struct PortamentoNode {
    value_out: Option<Primitive>,
    engaged: bool,
    active: bool,
    speed: f32,
    ramp: Ramp,
}

impl PortamentoNode {
    pub fn new(sound_config: &SoundConfig) -> Self {
        PortamentoNode {
            value_out: None,
            engaged: true,
            active: true,
            speed: 0.2,
            ramp: Ramp::new_with_start_value(sound_config, 440.0),
        }
    }
}

impl Node for PortamentoNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        if let Some(ramp_type) = state.props.get("ramp_type") {
            let ramp_type = ramp_type.clone().as_multiple_choice().unwrap();

            match ramp_type.as_str() {
                "exponential" => self.ramp.set_ramp_type(RampType::Exponential).unwrap(),
                "linear" => self.ramp.set_ramp_type(RampType::Linear).unwrap(),
                _ => {}
            };
        }

        InitResult::simple(vec![
            NodeRow::Property(
                "ramp_type".into(),
                PropertyType::MultipleChoice(vec!["exponential".into(), "linear".into()]),
                Property::MultipleChoice("exponential".into()),
            ),
            NodeRow::ValueInput(ValueSocketType::Gate, Primitive::Boolean(false), false),
            NodeRow::ValueInput(ValueSocketType::Frequency, Primitive::Float(440.0), false),
            NodeRow::ValueInput(ValueSocketType::Speed, Primitive::Float(0.2), false),
            NodeRow::ValueOutput(ValueSocketType::Frequency, Primitive::Float(440.0), false),
        ])
    }

    fn process(&mut self, _state: NodeProcessState, _streams_in: &[f32], _streams_out: &mut [f32]) -> NodeResult<()> {
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
                if !self.engaged {
                    self.value_out = Some(Primitive::Float(self.ramp.get_to()));
                    self.ramp.set_position(self.ramp.get_to());
                }
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
