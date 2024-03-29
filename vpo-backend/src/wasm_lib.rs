use std::{
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use futures::executor::block_on;
use ipc::{ipc_message::IpcMessage, send_buffer::SendBuffer};
use js_sys::Uint8Array;
use node_engine::{
    errors::NodeError,
    global_state::{GlobalState, Resources},
    state::NodeState,
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
    resource::{rank::parse_rank, sample::load_pipe, wavetable::load_wavetable},
    routes::route,
    utils::set_panic_hook,
};

pub fn get_midi(midi: Vec<u8>, parser: &mut MidiParser) -> Vec<MidiData> {
    let mut messages: Vec<MidiData> = Vec::new();

    parser.write(&midi).unwrap();

    while !parser.parsed.is_empty() {
        let message = parser.parsed.remove(0);
        messages.push(message);
    }

    messages
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct State {
    global_state: GlobalState,
    engine_state: NodeState,
    midi_parser: MidiParser,
}

#[wasm_bindgen]
impl State {
    pub fn new(sample_rate: u32, buffer_size: usize) -> State {
        let sound_config = SoundConfig {
            sample_rate,
            buffer_size,
        };

        set_panic_hook();

        let global_state = GlobalState {
            active_project: None,
            sound_config: sound_config,
            resources: Arc::new(RwLock::new(Resources::default())),
            import_folder: None,
        };

        let engine_state = NodeState::new(&global_state).unwrap();

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
            let result = block_on(route(
                IpcMessage::Json(serde_json::from_str(&message_in).unwrap()),
                &to_server,
                &mut self.engine_state,
                &mut self.global_state,
            ));

            match result {
                Ok(_) => {}
                Err(err) => {
                    let err_str = err.to_string();

                    to_server
                        .send(IpcMessage::Json(json! {{
                            "action": "toast/error",
                            "payload": err_str
                        }}))
                        .unwrap();
                }
            }
        }

        let mut midi = get_midi(midi_in.to_vec(), &mut self.midi_parser);

        self.engine_state
            .step(SmallVec::from(midi.clone()), &self.global_state, audio_out);

        let mut responses = to_server.buffer.lock().unwrap();

        let responses_json: Vec<Value> = responses
            .iter()
            .map(|IpcMessage::Json(response)| response.clone())
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

        let parent = path.iter().next().expect("resource needs to be in directory");
        let key = path.iter().skip(1).collect::<PathBuf>().to_string_lossy().to_string();

        match parent.to_str().unwrap() {
            "ranks" => {
                let config = String::from_utf8_lossy(&resource.to_vec()).into_owned();
                let rank = parse_rank(&config).context(LoadingSnafu)?;

                self.global_state.resources.ranks.add_resource(key, rank);
            }
            "pipes" => {
                let config = String::from_utf8_lossy(&resource.to_vec()).into_owned();
                let sample = associated_resource.map(|sample| sample.to_vec());

                let pipe = load_pipe(config, sample)?;

                self.global_state.resources.pipes.add_resource(key, pipe);
            }
            "wavetables" => {
                let wavetable = load_wavetable(resource.to_vec())?;

                self.global_state.resources.wavetables.add_resource(key, wavetable);
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
            Err(err) => Some(format!("{:?}", err)),
        }
    }

    fn load_with_error(&mut self, state: String) -> Result<(), NodeError> {
        let mut json: Value = serde_json::from_str(&state).context(JsonParserSnafu)?;

        self.engine_state = NodeEngineState::new(&self.global_state, self.engine_state.get_buffer_size()).unwrap();
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
