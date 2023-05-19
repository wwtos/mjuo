use resource_manager::{ResourceId, ResourceIndex};
use sound_engine::{sampling::interpolate::lerp, SoundConfig};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct WavetableSequencerNode {
    value_out: f32,
    /// 0-1, not 0-TWO_PI
    phase: f32,
    frequency: f32,
    advance_by: f32,
    index: Option<ResourceIndex>,
}

impl NodeRuntime for WavetableSequencerNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        if let Some(resource) = state.props.get("wavetable").and_then(|x| x.clone().as_resource()) {
            let new_index = state
                .resources
                .samples
                .get_index(&resource.resource)
                .ok_or(NodeError::MissingResource { resource })?;

            self.index = Some(new_index);
        }

        InitResult::nothing()
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        if let Some(index) = &mut self.index {
            let wavetable = &state.resources.samples.borrow_resource(*index).unwrap().audio_raw;

            let wavetable_pos = self.phase * wavetable.len() as f32;

            let wavetable_index = wavetable_pos as usize;
            let wavetable_offset = wavetable_pos - wavetable_pos.trunc();

            self.value_out = lerp(
                wavetable[wavetable_index],
                wavetable[(wavetable_index + 1) % wavetable.len()],
                wavetable_offset,
            );

            self.phase += self.advance_by * self.frequency;
            self.phase = self.phase % 1.0;
        }

        NodeOk::no_warnings(())
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let Some(frequency) = values_in[0].as_ref().and_then(|x| x.as_float()) {
            self.frequency = frequency;
        }
    }

    fn get_value_outputs(&mut self, values_out: &mut [Option<Primitive>]) {
        values_out[0] = Some(Primitive::Float(self.value_out));
    }
}

impl Node for WavetableSequencerNode {
    fn new(config: &SoundConfig) -> Self {
        let advance_by = (config.buffer_size as f32) / (config.sample_rate as f32);

        WavetableSequencerNode {
            phase: 0.0,
            frequency: 1.0,
            value_out: 0.0,
            advance_by,
            index: None,
        }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            NodeRow::Property(
                "wavetable".into(),
                PropertyType::Resource("samples".into()),
                Property::Resource(ResourceId {
                    namespace: "samples".into(),
                    resource: "".into(),
                }),
            ),
            value_input(register("frequency"), Primitive::Float(2.0)),
            value_output(register("value")),
        ])
    }
}
