use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use ipc::ipc_message::IPCMessage;
use sound_engine::constants::{BUFFER_SIZE, SAMPLE_RATE};
use sound_engine::SoundConfig;

use sound_engine::backend::{pulse::PulseClientBackend, AudioClientBackend};
use sound_engine::node::mono_buffer_player::MonoBufferPlayer;
use sound_engine::node::AudioNode;
use sound_engine::util::wav_reader::read_wav_as_mono;

use ipc::ipc_server::IPCServer;

fn start_ipc() -> (Sender<IPCMessage>, Receiver<IPCMessage>) {
    let (server_in, server_in_thread) = mpsc::channel::<IPCMessage>();
    let (server_out_thread, server_out) = mpsc::channel::<IPCMessage>();

    //thread::spawn(move || {
    IPCServer::open(server_in_thread, server_out_thread);
    //});

    (server_in, server_out)
}

fn connect_backend() -> Result<Box<dyn AudioClientBackend>, Box<dyn Error>> {
    let mut backend: Box<dyn AudioClientBackend> = Box::new(PulseClientBackend::new());
    backend.connect()?;

    Ok(backend)
}

fn main() -> Result<(), Box<dyn Error>> {
    start_ipc();

    let backend = connect_backend()?;

    let wav = read_wav_as_mono("./060-C.wav")?;
    let wav_wrapped = Rc::new(RefCell::new(wav));

    let config = SoundConfig {
        sample_rate: 48_000,
    };

    println!("loaded");

    let mut player = MonoBufferPlayer::new(&config, wav_wrapped);
    player.set_playback_rate(1.0);

    let mut buffer_index = 0;
    let start = Instant::now();

    // loop {
    //     let mut buffer = [0_f32; BUFFER_SIZE];

    //     for sample in buffer.iter_mut() {
    //         *sample = player.get_output_out();
    //         player.process();
    //     }

    //     backend.write(&buffer)?;

    //     let now = Instant::now() - start;
    //     let sample_duration =
    //         Duration::from_secs_f64(1.0 / (SAMPLE_RATE as f64 / BUFFER_SIZE as f64));
    //     let buffer_time =
    //         Duration::from_secs_f64((buffer_index as f64) * sample_duration.as_secs_f64());

    //     if !(now > buffer_time || buffer_time - now < Duration::from_secs_f64(0.3)) {
    //         thread::sleep(sample_duration);
    //     }

    //     buffer_index += 1;
    // }

    Ok(())
}
