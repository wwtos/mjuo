use resource_manager::{ResourceId, ResourceIndex};
use sound_engine::{node::wavetable_oscillator::WavetableOscillator, SoundConfig};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct WavetableNode {
    oscillator: Option<WavetableOscillator>,
    index: Option<ResourceIndex>,
    config: SoundConfig,
}

impl WavetableNode {
    pub fn new(config: &SoundConfig) -> Self {
        WavetableNode {
            oscillator: None,
            index: None,
            config: config.clone(),
        }
    }
}

impl NodeRuntime for WavetableNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let did_index_change;

        if let Some(resource) = state.props.get("sample").and_then(|x| x.clone().as_resource()) {
            let new_index = state
                .resources
                .samples
                .get_index(&resource.resource)
                .ok_or(NodeError::MissingResource { resource })?;

            did_index_change = Some(new_index) != self.index;
            self.index = Some(new_index);
        } else {
            did_index_change = false;
        }

        if self.oscillator.is_none() || did_index_change {
            let wavetable = state.resources.samples.borrow_resource(self.index.unwrap());

            if let Some(wavetable) = wavetable {
                self.oscillator = Some(WavetableOscillator::new(self.config.clone(), &wavetable));
            }
        }

        InitResult::nothing()
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        _streams_in: &[&[f32]],
        streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        if let Some(player) = &mut self.oscillator {
            let wavetable = state.resources.samples.borrow_resource(self.index.unwrap()).unwrap();

            for frame in streams_out[0].iter_mut() {
                *frame = player.get_next_sample(wavetable);
            }
        }

        NodeOk::no_warnings(())
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let [frequency] = values_in {
            if let Some(oscillator) = &mut self.oscillator {
                oscillator.set_frequency(frequency.clone().and_then(|x| x.as_float()).unwrap());
            }
        }
    }
}

impl Node for WavetableNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            NodeRow::Property(
                "wavetable".into(),
                PropertyType::Resource("wavetables".into()),
                Property::Resource(ResourceId {
                    namespace: "wavetables".into(),
                    resource: "".into(),
                }),
            ),
            value_input(register("frequency"), Primitive::Float(440.0)),
            stream_output(register("audio"), 0.0),
        ])
    }
}
