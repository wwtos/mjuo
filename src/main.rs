use std::error::Error;
use std::{io::Write, thread};
use std::time::{Duration, Instant};

use sound_engine::constants::{BUFFER_SIZE, SAMPLE_RATE};

use sound_engine::node::wav_reader::WavReader;
use sound_engine::backend::{pulse::PulseClientBackend, AudioClientBackend};

fn connect_backend() -> Result<Box<dyn AudioClientBackend>, Box<dyn Error>> {
    let mut backend: Box<dyn AudioClientBackend> = Box::new(PulseClientBackend::new());
    backend.connect()?;

    Ok(backend)
}

fn main() -> Result<(), Box<dyn Error>> {
    let backend = connect_backend()?;

    let mut reader = WavReader::new();
    reader.open("./060-C.wav");

    let mut buffer_index = 0;
    let start = Instant::now();

    loop {
        let before = Instant::now();

        let mut buffer = [0_f32; BUFFER_SIZE];
        let mut sample_index = 0;

        for sample in buffer.iter_mut() {
            if reader.available()? == 0 {
                return Ok(()); // nice clean stop
            }

            let sample_read = reader.read_one_sample().unwrap();

            *sample = (sample_read[0] + sample_read[1]) / 2.0;

            sample_index += 1;
        }

        backend.write(&buffer)?;

        let now = Instant::now() - start;
        let sample_duration = Duration::from_secs_f64(1.0 / (SAMPLE_RATE as f64 / BUFFER_SIZE as f64));
        let buffer_time = Duration::from_secs_f64((buffer_index as f64) * sample_duration.as_secs_f64());

        if !(now > buffer_time || buffer_time - now < Duration::from_secs_f64(0.3)) {
            thread::sleep(sample_duration);
        }

        buffer_index += 1;
    }
}
