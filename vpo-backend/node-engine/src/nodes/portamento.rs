use sound_engine::{
    node::ramp::{Ramp, RampType},
    SoundConfig,
};

use crate::{
    connection::{Primitive, ValueSocketType},
    errors::{NodeError, NodeOk},
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
            NodeRow::ValueInput(ValueSocketType::Gate, Primitive::Boolean(false)),
            NodeRow::ValueInput(ValueSocketType::Frequency, Primitive::Float(440.0)),
            NodeRow::ValueInput(ValueSocketType::Speed, Primitive::Float(0.2)),
            NodeRow::ValueOutput(ValueSocketType::Frequency, Primitive::Float(440.0)),
        ])
    }

    fn process(&mut self, _state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
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

    fn accept_value_input(&mut self, socket_type: &ValueSocketType, value: Primitive) {
        match socket_type {
            ValueSocketType::Gate => {
                self.engaged = value.as_boolean().unwrap();

                if !self.engaged {
                    self.value_out = Some(Primitive::Float(self.ramp.get_to()));
                    self.ramp.set_position(self.ramp.get_to());
                }
            }
            ValueSocketType::Frequency => {
                if self.engaged {
                    self.ramp
                        .set_ramp_parameters(self.ramp.get_position(), value.as_float().unwrap(), self.speed)
                        .unwrap();
                } else {
                    self.value_out = Some(Primitive::Float(value.as_float().unwrap()));
                }
            }
            ValueSocketType::Speed => {
                self.speed = value.as_float().unwrap();
                self.ramp
                    .set_ramp_parameters(self.ramp.get_position(), self.ramp.get_to(), self.speed)
                    .unwrap();
            }
            _ => {}
        }

        self.active = true;
    }

    fn get_value_output(&self, _socket_type: &ValueSocketType) -> Option<Primitive> {
        self.value_out.clone()
    }
}
