use std::error::Error;
use std::fs::File;
use std::thread;
use std::time::{Duration, Instant};

use node_engine::global_state::GlobalState;

use node_engine::state::NodeState;
use smallvec::SmallVec;
use sound_engine::midi::parse::MidiParser;
use sound_engine::SoundConfig;
use vpo_backend::io::cpal::CpalBackend;
use vpo_backend::{handle_msg, start_ipc, write_to_file};

fn main() -> Result<(), Box<dyn Error>> {
    // first, start ipc server
    let (to_server, from_server) = start_ipc();

    // create a global state
    let mut global_state = GlobalState::new(SoundConfig::default());

    // start up audio
    let mut backend = CpalBackend::new();
    let output_device = backend.get_default_output().unwrap();

    // can we get this far without crashing?
    backend.connect(output_device, global_state.resources.clone()).unwrap();

    Ok(())

    // let mut backend = connect_backend()?;
    // let mut midi_backend = connect_midi_backend()?;
    // let mut midi_parser = MidiParser::new();

    // let mut buffer_index = 0;
    // let start = Instant::now();

    // let mut output_file = File::create("out.pcm").unwrap();

    // loop {
    //     let msg = from_server.try_recv();

    //     if let Ok(msg) = msg {
    //         handle_msg(msg, &to_server, &mut engine_state, &mut global_state);
    //     }

    //     let mut midi = get_midi(&mut midi_backend, &mut midi_parser);

    //     if !midi.is_empty() {
    //         //println!("midi in main: {:?}", midi);
    //     }

    //     let mut buffer = vec![0_f32; buffer_size];

    //     for (i, sample) in buffer.iter_mut().enumerate() {
    //         let current_time = (buffer_index * buffer_size + i) as i64;

    //         *sample = engine_state.step(current_time, SmallVec::from(midi.clone()), &global_state);

    //         if !midi.is_empty() {
    //             midi = Vec::new();
    //         }
    //     }

    //     print!(", {:?}", buffer[0]);

    //     backend.write(&buffer)?;
    //     write_to_file(&mut output_file, &buffer)?;

    //     let now = Instant::now() - start;
    //     let sample_duration = Duration::from_secs_f64(buffer_size as f64 / SAMPLE_RATE as f64);
    //     let buffer_time = Duration::from_secs_f64((buffer_index as f64) * sample_duration.as_secs_f64());

    //     // println!("now: {:?}, now (buffer): {:?}", now, buffer_time);

    //     if !(now > buffer_time || buffer_time - now < sample_duration * 2) {
    //         thread::sleep(sample_duration);
    //     }

    //     buffer_index += 1;
    // }
}
