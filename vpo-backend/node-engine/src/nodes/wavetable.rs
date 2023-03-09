use resource_manager::{ResourceId, ResourceIndex};
use sound_engine::{node::wavetable_oscillator::WavetableOscillator, SoundConfig};

use crate::{
    connection::{Primitive, StreamSocketType, ValueSocketType},
    errors::{NodeError, NodeOk, NodeResult},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
    property::{Property, PropertyType},
};

#[derive(Debug, Clone)]
pub struct WavetableNode {
    oscillator: Option<WavetableOscillator>,
    index: ResourceIndex,
    config: SoundConfig,
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
        }
    }
}

impl Node for WavetableNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        let did_index_change;

        if let Some(resource) = state.props.get("wavetable").and_then(|x| x.clone().as_resource()) {
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
            let wavetable = state.global_state.resources.wavetables.borrow_resource(self.index);

            if let Some(wavetable) = wavetable {
                self.oscillator = Some(WavetableOscillator::new(self.config.clone(), &wavetable));
            }
        }

        InitResult::simple(vec![
            NodeRow::Property(
                "wavetable".into(),
                PropertyType::Resource("wavetables".into()),
                Property::Resource(ResourceId {
                    namespace: "wavetables".into(),
                    resource: "".into(),
                }),
            ),
            NodeRow::ValueInput(ValueSocketType::Frequency, Primitive::Float(440.0), false),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0, false),
        ])
    }

    fn process(&mut self, state: NodeProcessState, streams_in: &[f32], streams_out: &mut [f32]) -> NodeResult<()> {
        if let Some(player) = &mut self.oscillator {
            let wavetable = state
                .global_state
                .resources
                .wavetables
                .borrow_resource(self.index)
                .unwrap();

            streams_out[0] = player.get_next_sample(wavetable);
        }

        NodeOk::no_warnings(())
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        let [frequency] = values_in;

        if let Some(oscillator) = &mut self.oscillator {
            oscillator.set_frequency(frequency.and_then(|x| x.as_float()).unwrap());
        }
    }
}
