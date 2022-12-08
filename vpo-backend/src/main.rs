use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};

use node_engine::global_state::GlobalState;

use node_engine::state::NodeEngineState;
use smallvec::SmallVec;
use sound_engine::constants::{BUFFER_SIZE, SAMPLE_RATE};
use sound_engine::midi::parse::MidiParser;
use sound_engine::SoundConfig;
use vpo_backend::{connect_backend, connect_midi_backend, get_midi, handle_msg, start_ipc};

fn main() -> Result<(), Box<dyn Error>> {
    // first, start ipc server
    let (to_server, from_server) = start_ipc();

    // set up state
    let sound_config = SoundConfig {
        sample_rate: SAMPLE_RATE,
    };

    let mut global_state = GlobalState::new(sound_config);
    let mut engine_state = NodeEngineState::new(&global_state);

    let mut backend = connect_backend()?;
    let mut midi_backend = connect_midi_backend()?;
    let mut midi_parser = MidiParser::new();

    let mut buffer_index = 0;
    let start = Instant::now();

    let mut is_first_time = true;

    loop {
        let msg = from_server.try_recv();

        if let Ok(msg) = msg {
            handle_msg(msg, &to_server, &mut engine_state, &mut global_state);

            // TODO: this shouldn't reset `is_first_time` for just any message
            is_first_time = true;
        }

        let mut midi = get_midi(&mut midi_backend, &mut midi_parser);

        if !midi.is_empty() {
            //println!("midi in main: {:?}", midi);
        }

        let mut buffer = [0_f32; BUFFER_SIZE];

        for (i, sample) in buffer.iter_mut().enumerate() {
            let current_time = (buffer_index * BUFFER_SIZE + i) as i64;

            *sample = engine_state.step(current_time, is_first_time, SmallVec::from(midi.clone()), &global_state);

            if !midi.is_empty() {
                midi = Vec::new();
            }

            is_first_time = false;
        }

        backend.write(&buffer)?;
        //write_to_file(&mut output_file, &buffer)?;

        let now = Instant::now() - start;
        let sample_duration = Duration::from_secs_f64(BUFFER_SIZE as f64 / SAMPLE_RATE as f64);
        let buffer_time = Duration::from_secs_f64((buffer_index as f64) * sample_duration.as_secs_f64());

        // println!("now: {:?}, now (buffer): {:?}", now, buffer_time);

        if !(now > buffer_time || buffer_time - now < sample_duration * 2) {
            thread::sleep(sample_duration);
        }

        buffer_index += 1;
    }
}
