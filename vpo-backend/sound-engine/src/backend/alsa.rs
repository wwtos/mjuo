use std::{
    error,
    sync::mpsc::{channel, Sender},
    thread,
    time::{Duration, Instant},
};

use crate::constants::{BUFFER_SIZE, SAMPLE_RATE};
use alsa::{
    pcm::{Access, Format, HwParams},
    Direction, ValueOr, PCM,
};

use super::AudioClientBackend;

pub struct AlsaAudioBackend {
    sender: Option<Sender<[f32; BUFFER_SIZE]>>,
}

impl AlsaAudioBackend {
    pub fn new() -> Self {
        AlsaAudioBackend { sender: None }
    }
}

impl AudioClientBackend for AlsaAudioBackend {
    fn connect(&mut self) -> Result<(), Box<dyn error::Error>> {
        let (sender, receiver) = channel::<[f32; BUFFER_SIZE]>();

        thread::spawn(move || {
            loop {
                let pcm = PCM::new("default", Direction::Playback, false).unwrap();

                // Set hardware parameters: 48000 Hz / Mono / 16 bit
                let hwp = HwParams::any(&pcm).unwrap();
                hwp.set_channels(1).unwrap();
                hwp.set_rate(SAMPLE_RATE, ValueOr::Nearest).unwrap();
                hwp.set_format(Format::float()).unwrap();
                hwp.set_access(Access::RWInterleaved).unwrap();

                pcm.hw_params(&hwp).unwrap();

                // Make sure we don't start the stream too early
                let hwp = pcm.hw_params_current().unwrap();
                let swp = pcm.sw_params_current().unwrap();
                swp.set_start_threshold(hwp.get_buffer_size().unwrap()).unwrap();
                pcm.sw_params(&swp).unwrap();

                let io = pcm.io_f32().unwrap();
                let mut underrun_count = 0;

                let start = Instant::now();
                let mut buffer_index = 0;
                let sample_duration = Duration::from_secs_f64(BUFFER_SIZE as f64 / SAMPLE_RATE as f64);

                let mut first_time = true;

                'inner: loop {
                    // how much time do we have before we need to receive a message?
                    let buffer_time = buffer_index * sample_duration;
                    let now = Instant::now() - start;

                    let audio = if first_time {
                        first_time = false;
                        Ok(receiver.recv().unwrap())
                    } else {
                        let time_available = if buffer_time > now {
                            buffer_time - now
                        } else {
                            Duration::ZERO
                        };

                        if time_available < sample_duration * 2 {
                            receiver.recv_timeout(Duration::ZERO)
                        } else {
                            receiver.recv_timeout(time_available - sample_duration * 2)
                        }
                    };

                    let res = match audio {
                        Ok(audio) => {
                            if underrun_count == 0 {
                                buffer_index += 1;
                                io.writei(&audio)
                            } else {
                                underrun_count -= 1;
                                Ok(0)
                            }
                        }
                        Err(_) => {
                            println!("buffer underrun");
                            underrun_count += 1;
                            buffer_index += 1;

                            io.writei(&[0.0; BUFFER_SIZE])
                        }
                    };

                    match res {
                        Ok(_) => {}
                        Err(_) => break 'inner,
                    }
                }
            }
        });

        self.sender = Some(sender);

        Ok(())
    }

    fn write(&mut self, data: &[f32; BUFFER_SIZE]) -> Result<(), Box<dyn error::Error>> {
        self.sender.as_ref().unwrap().send(*data)?;

        Ok(())
    }

    fn drain(&self) -> Result<(), Box<dyn error::Error>> {
        unimplemented!();
    }
}
