use std::{
    collections::VecDeque,
    error,
    sync::{mpsc, Arc, RwLock},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, SampleFormat, SampleRate, Stream, StreamConfig,
};
use node_engine::{
    connection::MidiBundle,
    engine::NodeEngine,
    global_state::Resources,
    state::{FromNodeEngine, NodeEngineUpdate},
};
use rtrb::RingBuffer;
use smallvec::smallvec;
use snafu::{OptionExt, ResultExt};
use sound_engine::midi::messages::MidiMessage;
use tokio::sync::broadcast;

use crate::errors::EngineError;

pub struct CpalBackend {
    device: Option<Device>,
    host: Host,
}

impl CpalBackend {
    pub fn new() -> CpalBackend {
        let host = cpal::default_host();

        CpalBackend { device: None, host }
    }

    pub fn get_output_device_list(&self) -> Result<Vec<Device>, EngineError> {
        Ok(self
            .host
            .output_devices()
            .whatever_context("Could not enumerate devices")?
            .collect())
    }

    pub fn get_default_output(&self) -> Result<Device, EngineError> {
        self.host
            .default_output_device()
            .whatever_context("No default output device")
    }

    pub fn connect(
        &mut self,
        device: Device,
        resources: Arc<RwLock<Resources>>,
        buffer_size: usize,
        io_requested_buffer_size: usize,
        sample_rate: u32,
        midi_in: mpsc::Receiver<MidiBundle>,
        state_update_in: mpsc::Receiver<Vec<NodeEngineUpdate>>,
        to_main: broadcast::Sender<FromNodeEngine>,
    ) -> Result<(Stream, StreamConfig), EngineError> {
        let configs = device.supported_output_configs();

        let config_bounds = configs
            .whatever_context("Could not list supported output configs")?
            .find(|output_config| {
                output_config.max_sample_rate() >= SampleRate(sample_rate)
                    && output_config.min_sample_rate() <= SampleRate(sample_rate)
                    && output_config.channels() == 2
                    && output_config.sample_format() == SampleFormat::F32
            })
            .whatever_context("Could not build output config")?
            .with_sample_rate(SampleRate(sample_rate));

        println!("supported: {:?}", config_bounds);

        let config = StreamConfig {
            channels: config_bounds.channels(),
            sample_rate: config_bounds.sample_rate(),
            buffer_size: cpal::BufferSize::Fixed(io_requested_buffer_size as u32),
        };

        println!("Config: {:?}", config);
        let stream = self.build_output_callback(
            config.clone(),
            device,
            resources,
            buffer_size,
            config.sample_rate.0,
            midi_in,
            state_update_in,
            to_main,
        )?;

        Ok((stream, config))
    }

    fn build_output_callback(
        &mut self,
        config: StreamConfig,
        device: Device,
        resources: Arc<RwLock<Resources>>,
        buffer_size: usize,
        sample_rate: u32,
        midi_in: mpsc::Receiver<MidiBundle>,
        state_update_in: mpsc::Receiver<Vec<NodeEngineUpdate>>,
        to_main: broadcast::Sender<FromNodeEngine>,
    ) -> Result<Stream, EngineError> {
        let mut engine: NodeEngine = NodeEngine::uninitialized();
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(buffer_size * 2 * config.channels as usize);

        let mut buffer = vec![0_f32; buffer_size];
        let mut midi_buffer: VecDeque<MidiMessage> = VecDeque::new();

        let mut midi_time_offset: i64 = 0;
        let mut playback_time = 0;

        let stream = device
            .build_output_stream(
                &config,
                // main callback
                move |out: &mut [f32], _info| {
                    if let Ok(new_state) = state_update_in.try_recv() {
                        engine.apply_state_updates(new_state);
                        playback_time = 0;
                        midi_time_offset = 0;
                    }

                    if engine.initialized() {
                        // timing stuff (not fun)
                        let playback_time_micros = ((playback_time as f64 / sample_rate as f64) * 1_000_000f64) as i64;
                        let resources = resources.try_read();

                        if let Ok(resources) = resources {
                            for (i, frame) in out.iter_mut().enumerate() {
                                // are there enough slots open to step the engine?
                                if producer.slots() > buffer_size * config.channels as usize {
                                    let midi = midi_in.try_recv().unwrap_or(smallvec![]);

                                    if let Some(message) = midi.first() {
                                        if midi_time_offset == 0 || message.timestamp < playback_time_micros {
                                            midi_time_offset = playback_time_micros - message.timestamp as i64;
                                            println!("new offset: {}", midi_time_offset);
                                        }
                                    }

                                    midi_buffer.extend(midi.into_iter().map(|message| MidiMessage {
                                        data: message.data,
                                        timestamp: ((message.timestamp + midi_time_offset) * sample_rate as i64)
                                            / 1_000_000,
                                    }));

                                    // figure out how far before the midi messages exceed the current buffer time
                                    let from_engine = match midi_buffer
                                        .iter()
                                        .position(|message| message.timestamp > playback_time + buffer_size as i64)
                                    {
                                        Some(stop_at) => {
                                            let midi_constrained: MidiBundle = midi_buffer.drain(..stop_at).collect();
                                            engine.step(midi_constrained, &resources, &mut buffer)
                                        }
                                        None => {
                                            let midi_all: MidiBundle = midi_buffer.drain(..).collect();
                                            engine.step(midi_all, &resources, &mut buffer)
                                        }
                                    };

                                    if let Some(from_engine) = from_engine {
                                        to_main.send(from_engine).unwrap();
                                    }

                                    for buffer_frame in &buffer {
                                        for _ in 0..config.channels {
                                            producer.push(*buffer_frame).unwrap();
                                        }
                                    }
                                }

                                *frame = consumer.pop().unwrap();

                                if i % config.channels as usize == 0 {
                                    playback_time += 1;
                                }
                            }
                        } else {
                            // if we were unable to acquire resources in time, we'll
                            // just do nothing
                        }
                    }
                },
                |err| eprintln!("Callback error in cpal: {}", err),
                None,
            )
            .whatever_context("Failed to build output stream")?;
        stream.play().whatever_context("Could not start stream")?;

        Ok(stream)
    }
}

fn _enumerate_devices() -> Result<(), Box<dyn error::Error>> {
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
