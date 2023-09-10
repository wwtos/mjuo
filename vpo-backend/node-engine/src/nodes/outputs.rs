use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct OutputsNode {
    values: Vec<Option<Primitive>>,
    midis: Vec<Option<MidiBundle>>,
    streams: Vec<Vec<f32>>,
}

impl OutputsNode {
    pub fn get_streams(&self) -> &Vec<Vec<f32>> {
        &self.streams
    }

    pub fn get_midis(&self) -> &Vec<Option<MidiBundle>> {
        &self.midis
    }

    pub fn get_values(&self) -> &Vec<Option<Primitive>> {
        &self.values
    }
}

impl NodeRuntime for OutputsNode {
    fn process(
        &mut self,
        _globals: NodeProcessGlobals,
        ins: Ins,
        _outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        for (i, midi) in ins.midis.iter().enumerate() {
            if let Some(midi) = midi {
                self.midis[i] = Some(midi.clone());
            }
        }

        for (i, value) in ins.values.iter().enumerate() {
            if let Some(value) = value {
                self.values[i] = Some(value.clone());
            }
        }

        let buffer_size = ins.streams[0].len();

        self.streams.resize_with(ins.streams.len(), || vec![0.0; buffer_size]);

        for (local_stream, stream_in) in self.streams.iter_mut().zip(ins.streams) {
            local_stream.resize(buffer_size, 0.0);
            local_stream.copy_from_slice(stream_in);
        }

        NodeOk::no_warnings(())
    }

    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        if let Some(Property::SocketList(sockets)) = params.props.get("socket_list") {
            let midi_outputs = sockets
                .iter()
                .filter(|output| output.socket_type() == SocketType::Midi)
                .count();

            let value_outputs = sockets
                .iter()
                .filter(|output| output.socket_type() == SocketType::Value)
                .count();

            self.midis.resize(midi_outputs, None);
            self.values.resize(value_outputs, None);
        }

        InitResult::nothing()
    }
}

impl Node for OutputsNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        OutputsNode {
            values: vec![],
            midis: vec![],
            streams: vec![],
        }
    }

    fn get_io(props: HashMap<String, Property>, _register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        if let Some(Property::SocketList(sockets)) = props.get("socket_list") {
            NodeIo::simple(
                sockets
                    .iter()
                    .map(|socket_type| NodeRow::from_type_and_direction(socket_type.clone(), SocketDirection::Input))
                    .collect::<Vec<NodeRow>>(),
            )
        } else {
            NodeIo::simple(vec![])
        }
    }
}
