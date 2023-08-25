use resource_manager::{ResourceId, ResourceIndex};
use sound_engine::{node::wavetable_oscillator::WavetableOscillator, SoundConfig};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct WavetableNode {
    oscillator: WavetableOscillator,
    index: Option<ResourceIndex>,
}

impl NodeRuntime for WavetableNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let needed_resource = state.props.get("wavetable").and_then(|x| x.clone().as_resource());

        self.oscillator = WavetableOscillator::new(state.sound_config.clone());

        NodeOk::no_warnings(InitResult {
            changed_properties: None,
            needed_resources: needed_resource.map(|x| vec![x]).unwrap_or(vec![]),
        })
    }

    fn process(
        &mut self,
        globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        resources: &[(ResourceIndex, &dyn Any)],
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
    fn new(_config: &SoundConfig) -> Self {
        WavetableNode {
            oscillator: None,
            index: None,
        }
    }

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
            stream_output(register("audio")),
        ])
    }
}
