use std::{
    io::{Cursor, Write},
    path::PathBuf,
    sync::Mutex,
};

use futures::executor::block_on;
use ipc::{ipc_message::IPCMessage, send_buffer::SendBuffer};
use js_sys::Uint8Array;
use node_engine::{
    errors::{JsonParserSnafu, NodeError},
    global_state::{GlobalState, Resources},
    state::NodeEngineState,
};
use serde_json::{json, Value};
use smallvec::SmallVec;
use snafu::ResultExt;
use sound_engine::{
    midi::{messages::MidiData, parse::MidiParser},
    SoundConfig,
};
use wasm_bindgen::prelude::*;

use crate::{
    errors::{EngineError, LoadingSnafu},
    resource::{rank::load_rank, sample::load_pipe, wavetable::load_wavetable},
    routes::route,
    utils::set_panic_hook,
};

pub fn get_midi(midi: Uint8Array, parser: &mut MidiParser) -> Vec<MidiData> {
    let mut messages: Vec<MidiData> = Vec::new();

    for i in 0..midi.length() {
        parser.write(&[midi.at(i as i32).unwrap()]).unwrap();

        while !parser.parsed.is_empty() {
            let message = parser.parsed.remove(0);
            messages.push(message);
        }
    }

    messages
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct State {
    global_state: GlobalState,
    engine_state: NodeEngineState,
    midi_parser: MidiParser,
}

#[wasm_bindgen]
impl State {
    pub fn new(sample_rate: u32) -> State {
        let sound_config = SoundConfig { sample_rate };

        set_panic_hook();

        let global_state = GlobalState {
            active_project: None,
            sound_config: sound_config,
            resources: Resources::default(),
        };

        let engine_state = NodeEngineState::new(&global_state).unwrap();

        let midi_parser = MidiParser::new();

        State {
            global_state,
            engine_state,
            midi_parser,
        }
    }

    pub fn step(&mut self, message_in: Option<String>, midi_in: Uint8Array, audio_out: &mut [f32]) -> String {
        let to_server = SendBuffer {
            buffer: Mutex::new(vec![]),
        };

        if let Some(message_in) = message_in {
            let result = route(
                IPCMessage::Json(serde_json::from_str(&message_in).unwrap()),
                &to_server,
                &mut self.engine_state,
                &mut self.global_state,
            );

            match result {
                Ok(_) => {}
                Err(err) => {
                    let err_str = err.to_string();

                    block_on(async {
                        to_server
                            .send(IPCMessage::Json(json! {{
                                "action": "toast/error",
                                "payload": err_str
                            }}))
                            .await
                    });
                }
            }
        }

        let mut midi = get_midi(midi_in, &mut self.midi_parser);

        for sample in audio_out.iter_mut() {
            *sample = self.engine_state.step(SmallVec::from(midi.clone()), &self.global_state);

            if !midi.is_empty() {
                midi = Vec::new();
            }
        }

        let mut responses = to_server.buffer.lock().unwrap();

        let responses_json: Vec<Value> = responses
            .iter()
            .map(|IPCMessage::Json(response)| response.clone())
            .collect();

        responses.clear();

        serde_json::to_string(&responses_json).unwrap()
    }

    pub fn reset_state(&mut self) {
        self.global_state.reset();
    }

    fn load_resource_with_error(
        &mut self,
        path_raw: String,
        resource: Uint8Array,
        associated_resource: Option<Uint8Array>,
    ) -> Result<(), EngineError> {
        let path = PathBuf::from(path_raw);

        let parent = path.parent().unwrap();

        match parent.to_str().unwrap() {
            "ranks" => {
                let co
                let rank = load_rank(&mut resource.to_vec().as_slice()).context(LoadingSnafu)?;

                self.global_state.resources.ranks.add_resource(rank);
            }
            "pipes" => {
                let associated_resource =
                    associated_resource.map(|config_raw| Box::new(Cursor::new(resource.to_vec())));

                let sample = load_pipe(config, associated_resource)?;

                self.global_state.resources.samples.add_resource(sample);
            }
            "wavetables" => {
                let wavetable = load_wavetable(Box::new(Cursor::new(resource.to_vec())))?;

                self.global_state.resources.wavetables.add_resource(wavetable);
            }
            _ => {}
        };

        Ok(())
    }

    pub fn load_resource(
        &mut self,
        path_raw: String,
        resource: Uint8Array,
        config: Option<Uint8Array>,
    ) -> Option<String> {
        match self.load_resource_with_error(path_raw, resource, config) {
            Ok(()) => None,
            Err(err) => Some(err.to_string()),
        }
    }

    fn load_with_error(&mut self, state: String) -> Result<(), NodeError> {
        let mut json: Value = serde_json::from_str(&state).context(JsonParserSnafu)?;

        self.engine_state = NodeEngineState::new(&self.global_state).unwrap();
        self.engine_state.apply_json(json["state"].take(), &self.global_state)?;

        Ok(())
    }

    pub fn load(&mut self, state: String) -> Option<String> {
        match self.load_with_error(state) {
            Ok(()) => None,
            Err(err) => Some(err.to_string()),
        }
    }
}
