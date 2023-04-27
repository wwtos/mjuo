use std::{
    error,
    sync::{mpsc, Arc, RwLock},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, SampleFormat, SampleRate, Stream, StreamConfig,
};
use node_engine::{
    engine::NodeEngine,
    global_state::{GlobalState, Resources},
};
use smallvec::SmallVec;
use snafu::{whatever, OptionExt, ResultExt};

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
        Ok(self
            .host
            .default_output_device()
            .whatever_context("No default output device")?)
    }

    pub fn connect(
        &mut self,
        device: Device,
        resources: Arc<RwLock<Resources>>,
    ) -> Result<mpsc::Sender<NodeEngine>, EngineError> {
        let configs = device.supported_output_configs();

        let config = configs
            .whatever_context("Could not list supported output configs")?
            .find(|output_config| {
                output_config.max_sample_rate() >= SampleRate(44_100)
                    && output_config.min_sample_rate() <= SampleRate(44_100)
                    && output_config.channels() >= 1
                    && output_config.sample_format() == SampleFormat::F32
            })
            .whatever_context("Could not build output config")?
            .with_sample_rate(SampleRate(44_100));

        println!("Config: {:?}", config);

        let (output_stream, sender) = match config.sample_format() {
            SampleFormat::F32 => self.build_output_callback(config.into(), device, resources)?,
            _ => whatever!("I'm just working with f32 today, thank you very much"),
        };

        println!("Successfully built streams.");

        output_stream.play().whatever_context("Could not start stream")?;

        Ok(sender)
    }

    fn build_output_callback(
        &mut self,
        config: StreamConfig,
        device: Device,
        resources: Arc<RwLock<Resources>>,
    ) -> Result<(Stream, mpsc::Sender<NodeEngine>), EngineError> {
        let (sender, receiver) = mpsc::channel();

        let mut engine: Option<NodeEngine> = None;

        let stream = device
            .build_output_stream::<f32, _, _>(
                &config,
                move |out, info| {
                    if let Ok(new_engine) = receiver.try_recv() {
                        engine = Some(new_engine);
                    }

                    if let Some(ref mut engine) = &mut engine {
                        let resources = resources.try_read();

                        // if we were unable to acquire resources in time, we'll
                        // just do nothing
                        if let Ok(resources) = resources {
                            engine.step(SmallVec::new(), &resources, out);
                        }
                    }
                },
                |err| panic!("Callback error! {}", err),
                None,
            )
            .whatever_context("Failed to build output stream")?;

        Ok((stream, sender))
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
