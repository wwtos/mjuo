use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate,
};
use ringbuf::{Producer, RingBuffer};
use std::error;

use super::{AudioClientBackend, BUFFER_SIZE};

pub struct CpalBackend {
    latency: f32,
    producer: Option<Producer<f32>>,
}

impl CpalBackend {
    pub fn new(latency: f32) -> Result<CpalBackend, Box<dyn error::Error>> {
        enumerate_devices()?;

        Ok(CpalBackend {
            latency,
            producer: None,
        })
    }
}

impl AudioClientBackend for CpalBackend {
    fn connect(&mut self) -> Result<(), Box<dyn error::Error>> {
        let host = cpal::default_host();

        let device = host.default_output_device().expect("failed to find output device");
        println!("Output device: {}", device.name()?);

        let config: cpal::StreamConfig = device
            .supported_output_configs()
            .unwrap()
            .find(|output_config| {
                output_config.max_sample_rate() >= SampleRate(44_100)
                    && output_config.min_sample_rate() <= SampleRate(44_100)
                    && output_config.channels() >= 1
                    && output_config.sample_format() == SampleFormat::F32
            })
            .unwrap()
            .with_sample_rate(SampleRate(44_100))
            .into();

        println!("Config: {:?}", config);

        let latency_frames = (self.latency / 1_000.0) * config.sample_rate.0 as f32;
        let latency_samples = latency_frames as usize * config.channels as usize;

        let ring = RingBuffer::new(latency_samples * 2);
        let (mut producer, mut consumer) = ring.split();

        // Fill the samples with 0.0 equal to the length of the delay.
        for _ in 0..latency_samples {
            // The ring buffer has twice as much space as necessary to add latency here,
            // so this should never fail
            producer.push(0.0).unwrap();
        }

        self.producer = Some(producer);

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut input_fell_behind = false;
            for sample in data {
                *sample = match consumer.pop() {
                    Some(s) => s,
                    None => {
                        input_fell_behind = true;
                        0.0
                    }
                };
            }
            if input_fell_behind {
                eprintln!("input stream fell behind: try increasing latency");
            }
        };

        let output_stream = device.build_output_stream(&config, output_data_fn, err_fn)?;
        println!("Successfully built streams.");

        println!(
            "Starting the input and output streams with `{}` milliseconds of latency.",
            self.latency
        );
        output_stream.play()?;

        Ok(())
    }

    fn write(&mut self, data: &[f32; BUFFER_SIZE]) -> Result<(), Box<dyn error::Error>> {
        let producer = self.producer.as_mut().unwrap();

        for elem in data.iter() {
            producer.push(*elem).unwrap();
        }

        Ok(())
    }

    fn drain(&self) -> Result<(), Box<dyn error::Error>> {
        todo!()
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn enumerate_devices() -> Result<(), Box<dyn error::Error>> {
    println!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
    let available_hosts = cpal::available_hosts();
    println!("Available hosts:\n  {:?}", available_hosts);

    for host_id in available_hosts {
        println!("{}", host_id.name());
        let host = cpal::host_from_id(host_id)?;

        let default_in = host.default_input_device().map(|e| e.name().unwrap());
        let default_out = host.default_output_device().map(|e| e.name().unwrap());
        println!("  Default Input Device:\n    {:?}", default_in);
        println!("  Default Output Device:\n    {:?}", default_out);

        let devices = host.devices()?;
        println!("  Devices: ");
        for (device_index, device) in devices.enumerate() {
            println!("  {}. \"{}\"", device_index + 1, device.name()?);

            // Input configs
            if let Ok(conf) = device.default_input_config() {
                println!("    Default input stream config:\n      {:?}", conf);
            }
            let input_configs = match device.supported_input_configs() {
                Ok(f) => f.collect(),
                Err(e) => {
                    println!("    Error getting supported input configs: {:?}", e);
                    Vec::new()
                }
            };
            if !input_configs.is_empty() {
                println!("    All supported input stream configs:");
                for (config_index, config) in input_configs.into_iter().enumerate() {
                    println!("      {}.{}. {:?}", device_index + 1, config_index + 1, config);
                }
            }

            // Output configs
            if let Ok(conf) = device.default_output_config() {
                println!("    Default output stream config:\n      {:?}", conf);
            }

            // let output_configs = match device.supported_output_configs() {
            //     Ok(f) => f.collect(),
            //     Err(e) => {
            //         println!("    Error getting supported output configs: {:?}", e);
            //         Vec::new()
            //     }
            // };
            // if !output_configs.is_empty() {
            //     println!("    All supported output stream configs:");
            //     for (config_index, config) in output_configs.into_iter().enumerate() {
            //         println!("      {}.{}. {:?}", device_index + 1, config_index + 1, config);
            //     }
            // }
        }
    }

    Ok(())
}
