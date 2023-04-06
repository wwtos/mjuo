use std::{io::Write, sync::Mutex};

use futures::executor::block_on;
use ipc::{ipc_message::IPCMessage, send_buffer::SendBuffer};
use js_sys::Uint8Array;
use node_engine::{
    global_state::{GlobalState, Resources},
    state::NodeEngineState,
};
use serde_json::json;
use smallvec::SmallVec;
use sound_engine::{
    midi::{messages::MidiData, parse::MidiParser},
    SoundConfig,
};
use wasm_bindgen::prelude::*;

use crate::{routes::route, utils::set_panic_hook};

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

        let responses = to_server.buffer.lock().unwrap();

        let response = match responses.first() {
            Some(message) => {
                let IPCMessage::Json(message) = message;
                serde_json::to_string(message).unwrap()
            }
            None => "".into(),
        };

        response
    }
}
