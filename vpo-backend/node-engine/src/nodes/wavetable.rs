use resource_manager::{ResourceId, ResourceIndex};
use sound_engine::{node::wavetable_oscillator::WavetableOscillator, MonoSample, SoundConfig};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct WavetableNode {
    oscillator: WavetableOscillator,
    index: Option<ResourceIndex>,
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

    fn process<'brand>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'_, 'brand>,
        outs: Outs<'_, 'brand>,
        token: &mut GhostToken<'brand>,
        resources: &[&dyn Any],
    ) -> NodeResult<()> {
        if let Some(frequency) = ins.values[0][0].borrow(token).as_float() {
            self.oscillator.set_frequency(frequency);
        }

        if let Some(wavetable) = resources[0].downcast_ref::<MonoSample>() {
            for frame in outs.streams[0][0].iter_mut() {
                *frame.borrow_mut(token) = self.oscillator.get_next_sample(wavetable);
            }
        }

        NodeOk::no_warnings(())
    }
}

impl Node for WavetableNode {
    fn new(config: &SoundConfig) -> Self {
        WavetableNode {
            oscillator: WavetableOscillator::new(config.clone()),
            index: None,
        }
    }

    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            NodeRow::Property(
                "wavetable".into(),
                PropertyType::Resource("wavetables".into()),
                Property::Resource(ResourceId {
                    namespace: "wavetables".into(),
                    resource: "".into(),
                }),
            ),
            value_input("frequency", Primitive::Float(440.0), 1),
            stream_output("audio", 1),
        ])
    }
}
