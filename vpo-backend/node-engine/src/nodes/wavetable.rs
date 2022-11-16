use resource_manager::ResourceIndex;
use sound_engine::{node::wavetable_oscillator::WavetableOscillator, SoundConfig};

use crate::{
    connection::{Primitive, StreamSocketType, ValueSocketType},
    errors::{NodeError, NodeOk},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
    property::{Property, PropertyType, Resource},
};

#[derive(Debug, Clone)]
pub struct WavetableNode {
    oscillator: Option<WavetableOscillator>,
    index: ResourceIndex,
    config: SoundConfig,
    output: f32,
}

impl WavetableNode {
    pub fn new(config: &SoundConfig) -> Self {
        WavetableNode {
            oscillator: None,
            index: ResourceIndex {
                index: 0,
                generation: 0,
            },
            config: config.clone(),
            output: 0.0,
        }
    }
}

impl Node for WavetableNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        let did_index_change;

        if let Some(Some(resource)) = state
            .props
            .get("wavetable")
            .map(|wavetable| wavetable.clone().as_resource())
        {
            let new_index = state
                .global_state
                .resources
                .wavetables
                .get_index(&resource.resource)
                .ok_or(NodeError::MissingResource { resource })?;

            did_index_change = new_index != self.index;
            self.index = new_index;
        } else {
            did_index_change = false;
        }

        if self.oscillator.is_none() || did_index_change {
            let wavetable = state
                .global_state
                .resources
                .wavetables
                .borrow_resource(self.index)
                .unwrap();

            self.oscillator = Some(WavetableOscillator::new(self.config.clone(), &wavetable));
        }

        InitResult::simple(vec![
            NodeRow::Property(
                "wavetable".into(),
                PropertyType::Resource("wavetables".into()),
                Property::Resource(Resource {
                    namespace: "wavetables".into(),
                    resource: "".into(),
                }),
            ),
            NodeRow::ValueInput(ValueSocketType::Frequency, Primitive::Float(440.0)),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0),
        ])
    }

    fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        if let Some(player) = &mut self.oscillator {
            let wavetable = state
                .global_state
                .resources
                .wavetables
                .borrow_resource(self.index)
                .unwrap();

            self.output = player.get_next_sample(wavetable);
        }

        NodeOk::no_warnings(())
    }

    fn accept_value_input(&mut self, socket_type: &ValueSocketType, value: Primitive) {
        if let Some(oscillator) = &mut self.oscillator {
            oscillator.set_frequency(value.as_float().unwrap());
        }
    }

    fn get_stream_output(&self, _socket_type: &StreamSocketType) -> f32 {
        self.output
    }
}
