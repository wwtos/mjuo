use core::fmt;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt::Debug,
    ops::Range,
    thread::JoinHandle,
};

use clocked::{
    cpal::{start_cpal_sink, start_cpal_source, CpalSink, CpalSource},
    midir::{start_midir_sink, start_midir_source, MidirSink, MidirSource},
};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host, HostId, SampleFormat, SampleRate, StreamConfig, SupportedStreamConfigRange,
};
use generational_arena::Index;
use log::trace;
use midir::{MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputPort};
use node_engine::io_routing::DeviceDirection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use snafu::{OptionExt, ResultExt};

use crate::errors::{DeviceNotInCpalListSnafu, DeviceStartSnafu, EngineError};

pub enum MidiDevice {
    Source(MidiInputPort),
    Sink(MidiOutputPort),
}

impl MidiDevice {
    pub fn device_type(&self) -> DeviceDirection {
        match self {
            MidiDevice::Source(_) => DeviceDirection::Source,
            MidiDevice::Sink(_) => DeviceDirection::Sink,
        }
    }
}

pub struct CpalDeviceStatus {
    pub sink_handle: Option<cpal::Stream>,
    pub source_handle: Option<cpal::Stream>,
    pub host_id: HostId,
    pub name: String,
    pub source_options: Option<StreamConfigOptions>,
    pub sink_options: Option<StreamConfigOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CpalJsonDeviceStatus {
    name: String,
    source_options: Option<StreamConfigOptions>,
    sink_options: Option<StreamConfigOptions>,
    source_taken: bool,
    sink_taken: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MidirJsonDeviceStatus {
    name: String,
    source_taken: bool,
    sink_taken: bool,
}

impl fmt::Debug for CpalDeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CpalDeviceStatus")
            .field("sink_handle", &self.sink_handle.as_ref().map(|_| "Stream { .. }"))
            .field("source_handle", &self.sink_handle.as_ref().map(|_| "Stream { .. }"))
            .field("host_id", &format!("HostId({})", self.host_id.name()))
            .field("name", &self.name)
            .field("input_options", &self.source_options)
            .field("output_options", &self.sink_options)
            .finish()
    }
}

pub struct MidirDeviceStatus {
    pub sink_handle: Option<JoinHandle<()>>,
    pub source_handle: Option<MidiInputConnection<()>>,
    pub name: String,
    pub device: MidiDevice,
}

impl Debug for MidirDeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MidirDeviceStatus")
            .field("sink_handle", &self.sink_handle)
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CpalIndex(pub Index);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MidirIndex(pub Index);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfigOptions {
    pub channels: Range<u16>,
    pub sample_rate: Range<u32>,
    pub buffer_size: Range<usize>,
    pub sample_formats: BTreeSet<MySampleFormat>,
}

pub struct DeviceManager {
    cpal_hosts: Vec<Host>,
    cpal_statuses: BTreeMap<String, CpalDeviceStatus>,
    midir_statuses: BTreeMap<String, MidirDeviceStatus>,
    midir_input_scan: MidiInput,
    midir_output_scan: MidiOutput,
}

pub struct ScanResult {
    pub cpal_added: Vec<String>,
    pub cpal_removed: Vec<String>,
    pub midir_added: Vec<String>,
    pub midir_removed: Vec<String>,
}

impl Debug for DeviceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeviceManager { .. }")
    }
}

impl DeviceManager {
    pub fn new() -> DeviceManager {
        let hosts: Vec<_> = cpal::available_hosts()
            .into_iter()
            .filter_map(|host_id| cpal::host_from_id(host_id).ok())
            .collect();

        let mut midi_in = MidiInput::new("midi inputs scanner").unwrap();
        midi_in.ignore(midir::Ignore::None);
        let midi_out = MidiOutput::new("midi outputs scanner").unwrap();

        let mut manager = DeviceManager {
            cpal_hosts: hosts,
            cpal_statuses: BTreeMap::new(),
            midir_statuses: BTreeMap::new(),
            midir_input_scan: midi_in,
            midir_output_scan: midi_out,
        };

        manager.rescan_devices();

        manager
    }

    /// Rescans and returns a list of indexes of the new devices
    pub fn rescan_devices(&mut self) -> ScanResult {
        let (cpal_added, cpal_removed) = self.rescan_cpal_devices();
        let (midir_added, midir_removed) = self.rescan_midir_devices();

        ScanResult {
            cpal_added,
            cpal_removed,
            midir_added,
            midir_removed,
        }
    }

    fn rescan_midir_devices(&mut self) -> (Vec<String>, Vec<String>) {
        let mut midi_in = MidiInput::new("midi scan").unwrap();
        midi_in.ignore(midir::Ignore::None);
        let midi_out = MidiOutput::new("midi scan").unwrap();

        let mut current_sources: Vec<MidirDeviceStatus> = midi_in
            .ports()
            .into_iter()
            .map(|port| MidirDeviceStatus {
                name: midi_in.port_name(&port).unwrap(),
                device: MidiDevice::Source(port),
                sink_handle: None,
                source_handle: None,
            })
            .collect();
        let current_sinks: Vec<MidirDeviceStatus> = midi_out
            .ports()
            .into_iter()
            .map(|port| MidirDeviceStatus {
                name: midi_out.port_name(&port).unwrap(),
                device: MidiDevice::Sink(port),
                sink_handle: None,
                source_handle: None,
            })
            .collect();

        current_sources.extend(current_sinks.into_iter());
        let current_devices = current_sources;

        // this is a bit of a hack, but if it errors out when asking the device's config,
        // it's considered disconnected
        let mut to_remove = vec![];
        for (name, my_device) in self.midir_statuses.iter() {
            if !self
                .midir_statuses
                .iter()
                .any(|(_, x)| x.name == my_device.name && x.device.device_type() == my_device.device.device_type())
            {
                // couldn't find `my_device` in the new list, perhaps it's disconnected now?
                match &my_device.device {
                    MidiDevice::Source(source) => {
                        if self.midir_input_scan.port_name(&source).is_err() {
                            // definitely disconnected
                            to_remove.push(name.clone());
                        }
                    }
                    MidiDevice::Sink(sink) => {
                        if self.midir_output_scan.port_name(&sink).is_err() {
                            // definitely disconnected
                            to_remove.push(name.clone());
                        }
                    }
                }
            }
        }

        for name in to_remove.iter() {
            self.cpal_statuses.remove(name);
        }

        let mut new_indexes = vec![];

        for new_device in current_devices {
            if !self
                .midir_statuses
                .iter()
                .any(|(_, x)| x.name == new_device.name && x.device.device_type() == new_device.device.device_type())
            {
                trace!("tracking midi device: {:?}", new_device.name);

                new_indexes.push(new_device.name.clone());
                self.midir_statuses.insert(new_device.name.clone(), new_device);
            }
        }

        (new_indexes, to_remove)
    }

    fn rescan_cpal_devices(&mut self) -> (Vec<String>, Vec<String>) {
        let current_device_list: Vec<CpalDeviceStatus> = self
            .cpal_hosts
            .iter()
            .flat_map(|host| {
                let id = host.id();

                let devices = host.devices().unwrap().map(move |device| {
                    let is_source = device
                        .supported_input_configs()
                        .map(|mut x| x.any(|_| true))
                        .unwrap_or(false);
                    let is_sink = device
                        .supported_output_configs()
                        .map(|mut x| x.any(|_| true))
                        .unwrap_or(false);

                    let source_options = if is_source {
                        Some(DeviceManager::cpal_simplify_configs(
                            device.supported_input_configs().map(|x| x.collect()).unwrap_or(vec![]),
                        ))
                    } else {
                        None
                    };

                    let sink_options = if is_sink {
                        Some(DeviceManager::cpal_simplify_configs(
                            device.supported_output_configs().map(|x| x.collect()).unwrap_or(vec![]),
                        ))
                    } else {
                        None
                    };

                    CpalDeviceStatus {
                        host_id: id,
                        name: device.name().unwrap(),
                        source_options,
                        sink_options,
                        sink_handle: None,
                        source_handle: None,
                    }
                });

                devices
            })
            .collect();

        let mut new_indexes = vec![];

        for new_device in current_device_list {
            if !self.cpal_statuses.iter().any(|(_, x)| x.name == new_device.name) {
                trace!("tracking audio device: {:?}", new_device.name);

                new_indexes.push(new_device.name.clone());
                self.cpal_statuses.insert(new_device.name.clone(), new_device);
            }
        }

        (new_indexes, vec![])
    }

    pub fn devices_as_json(&self) -> serde_json::Value {
        let cpal: HashMap<String, CpalJsonDeviceStatus> = self
            .cpal_statuses
            .iter()
            .map(|(key, status)| {
                (
                    key.clone(),
                    CpalJsonDeviceStatus {
                        name: status.name.clone(),
                        source_options: status.source_options.clone(),
                        sink_options: status.sink_options.clone(),
                        source_taken: status.source_handle.is_some(),
                        sink_taken: status.sink_handle.is_some(),
                    },
                )
            })
            .collect();

        let midir: HashMap<String, MidirJsonDeviceStatus> = self
            .midir_statuses
            .iter()
            .map(|(key, status)| {
                (
                    key.clone(),
                    MidirJsonDeviceStatus {
                        name: status.name.clone(),
                        source_taken: status.source_handle.is_some(),
                        sink_taken: status.sink_handle.is_some(),
                    },
                )
            })
            .collect();

        json!({
            "streams": cpal,
            "midi": midir
        })
    }

    pub fn cpal_devices(&self) -> &BTreeMap<String, CpalDeviceStatus> {
        &self.cpal_statuses
    }

    pub fn cpal_get_device(&self, device: &str) -> Option<Device> {
        let status = self.cpal_statuses.get(device)?;
        let host = cpal::host_from_id(status.host_id).ok()?;

        host.devices()
            .ok()?
            .find(|x| x.name().unwrap_or_default() == status.name)
    }

    pub fn cpal_input_config_options(&self, index: &str) -> Option<StreamConfigOptions> {
        self.cpal_get_device(index)?
            .supported_input_configs()
            .ok()
            .map(|configs| {
                let collected: Vec<_> = configs.collect();

                DeviceManager::cpal_simplify_configs(collected)
            })
    }

    pub fn cpal_output_config_options(&self, index: &str) -> Option<StreamConfigOptions> {
        self.cpal_get_device(index)?
            .supported_output_configs()
            .ok()
            .map(|configs| {
                let collected: Vec<_> = configs.collect();

                DeviceManager::cpal_simplify_configs(collected)
            })
    }

    pub fn cpal_simplify_configs(configs: Vec<SupportedStreamConfigRange>) -> StreamConfigOptions {
        let min_channels = configs.iter().map(|x| x.channels()).min().unwrap_or(0) as u16;
        let mut max_channels = configs.iter().map(|x| x.channels()).max().unwrap_or(0) as u16;

        // if max_channels == 32 {
        //     max_channels = u16::MAX; // silly cpal
        // }

        let min_sample_rate = configs.iter().map(|x| x.min_sample_rate().0).min().unwrap_or(0);
        let max_sample_rate = configs.iter().map(|x| x.max_sample_rate().0).max().unwrap_or(0);

        let min_buffer_size = configs
            .iter()
            .filter_map(|x| match x.buffer_size() {
                cpal::SupportedBufferSize::Range { min, .. } => Some(*min),
                cpal::SupportedBufferSize::Unknown => None,
            })
            .min()
            .unwrap_or(1) as usize;
        let max_buffer_size = configs
            .iter()
            .filter_map(|x| match x.buffer_size() {
                cpal::SupportedBufferSize::Range { max, .. } => Some(*max),
                cpal::SupportedBufferSize::Unknown => None,
            })
            .min()
            .unwrap_or(48_000) as usize;

        let sample_formats: BTreeSet<MySampleFormat> = configs.iter().map(|x| x.sample_format().into()).collect();

        StreamConfigOptions {
            channels: min_channels..max_channels,
            sample_rate: min_sample_rate..max_sample_rate,
            buffer_size: min_buffer_size..max_buffer_size,
            sample_formats: sample_formats,
        }
    }

    pub fn cpal_start_sink(
        &mut self,
        device_name: &str,
        channels: u16,
        sample_rate: u32,
        buffer_size: u32,
        periods: usize,
    ) -> Result<CpalSink, EngineError> {
        if let Some(device) = self.cpal_statuses.get(device_name) {
            if device.sink_handle.is_some() {
                return Err(EngineError::DeviceAlreadyStarted {
                    device_name: device_name.to_string(),
                });
            }

            let device = self
                .cpal_get_device(device_name)
                .with_context(|| DeviceNotInCpalListSnafu {
                    device_name: device_name.to_string(),
                })?;

            let (handle, instance) = start_cpal_sink(
                &device,
                &StreamConfig {
                    channels: channels as u16,
                    sample_rate: SampleRate(sample_rate),
                    buffer_size: cpal::BufferSize::Fixed(buffer_size),
                },
                SampleFormat::F32,
                buffer_size as usize + 128,
            )
            .context(DeviceStartSnafu)?;

            self.cpal_statuses.get_mut(device_name).unwrap().sink_handle = Some(handle);

            Ok(instance)
        } else {
            Err(EngineError::DeviceDoesNotExist {
                device_name: device_name.into(),
            })
        }
    }

    pub fn cpal_start_source(
        &mut self,
        device_name: &str,
        channels: u16,
        sample_rate: u32,
        buffer_size: u32,
        periods: usize,
    ) -> Option<CpalSource> {
        if let Some(device) = self.cpal_statuses.get(device_name) {
            if device.source_handle.is_some() {
                return None;
            }

            let (handle, instance) = start_cpal_source(
                &self.cpal_get_device(device_name)?,
                &StreamConfig {
                    channels: channels as u16,
                    sample_rate: SampleRate(sample_rate),
                    buffer_size: cpal::BufferSize::Fixed(buffer_size),
                },
                SampleFormat::F32,
                buffer_size as usize + 128,
            )
            .ok()?;

            self.cpal_statuses.get_mut(device_name).unwrap().source_handle = Some(handle);

            Some(instance)
        } else {
            None
        }
    }

    pub fn cpal_stop_device(&mut self, index: &str, freeing_sink: bool, freeing_source: bool) {
        if let Some(device) = self.cpal_statuses.get_mut(index) {
            if freeing_sink {
                device.sink_handle = None; // drops sink handle
            }

            if freeing_source {
                device.source_handle = None; // drops source handle
            }
        }
    }

    pub fn midir_stop_device(&mut self, index: &str, freeing_sink: bool, freeing_source: bool) {
        if let Some(device) = self.midir_statuses.get_mut(index) {
            if freeing_sink {
                device.sink_handle = None; // drops sink handle
            }

            if freeing_source {
                device.source_handle = None; // drops source handle
            }
        }
    }

    pub fn reset(&mut self) {
        self.cpal_statuses.clear();
        self.midir_statuses.clear();
        self.rescan_devices();
    }

    pub fn midir_devices(&self) -> &BTreeMap<String, MidirDeviceStatus> {
        &self.midir_statuses
    }

    pub fn midir_start_sink(&mut self, port_name: &str) -> Option<MidirSink> {
        if self.midir_statuses.contains_key(port_name) {
            let sink_name = format!("mjuo_output_{port_name}");
            let sink = MidiOutput::new(&sink_name).ok()?;

            let ports = sink.ports();
            let port = ports
                .iter()
                .find(|port| sink.port_name(port).map(|x| x == port_name).unwrap_or(false))?;

            let (handle, sink) = start_midir_sink(sink, port, &sink_name).ok()?;

            self.midir_statuses.get_mut(port_name).unwrap().sink_handle = Some(handle);

            Some(sink)
        } else {
            None
        }
    }

    pub fn midir_start_source(&mut self, port_name: &str) -> Option<MidirSource> {
        if self.midir_statuses.contains_key(port_name) {
            let sink_name = format!("mjuo_input_{port_name}");
            let source = MidiInput::new(&sink_name).ok()?;

            let ports = source.ports();
            let port = ports
                .iter()
                .find(|port| source.port_name(port).map(|x| x == port_name).unwrap_or(false))?;

            let (handle, source) = start_midir_source(source, port, &sink_name).ok()?;

            self.midir_statuses.get_mut(port_name).unwrap().source_handle = Some(handle);

            Some(source)
        } else {
            None
        }
    }
}

pub struct DeviceState {}

const SAMPLE_TYPE_PREFERENCE: [MySampleFormat; 10] = [
    MySampleFormat::F32,
    MySampleFormat::F64,
    MySampleFormat::I64,
    MySampleFormat::U64,
    MySampleFormat::I32,
    MySampleFormat::U32,
    MySampleFormat::I16,
    MySampleFormat::U16,
    MySampleFormat::I8,
    MySampleFormat::U8,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum MySampleFormat {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl From<SampleFormat> for MySampleFormat {
    fn from(value: SampleFormat) -> Self {
        match value {
            SampleFormat::I8 => MySampleFormat::I8,
            SampleFormat::I16 => MySampleFormat::I16,
            SampleFormat::I32 => MySampleFormat::I32,
            SampleFormat::I64 => MySampleFormat::I64,
            SampleFormat::U8 => MySampleFormat::U8,
            SampleFormat::U16 => MySampleFormat::U16,
            SampleFormat::U32 => MySampleFormat::U32,
            SampleFormat::U64 => MySampleFormat::U64,
            SampleFormat::F32 => MySampleFormat::F32,
            SampleFormat::F64 => MySampleFormat::F64,
            _ => unreachable!("curse you cpal"),
        }
    }
}

impl Into<SampleFormat> for MySampleFormat {
    fn into(self) -> SampleFormat {
        match self {
            MySampleFormat::I8 => SampleFormat::I8,
            MySampleFormat::I16 => SampleFormat::I16,
            MySampleFormat::I32 => SampleFormat::I32,
            MySampleFormat::I64 => SampleFormat::I64,
            MySampleFormat::U8 => SampleFormat::U8,
            MySampleFormat::U16 => SampleFormat::U16,
            MySampleFormat::U32 => SampleFormat::U32,
            MySampleFormat::U64 => SampleFormat::U64,
            MySampleFormat::F32 => SampleFormat::F32,
            MySampleFormat::F64 => SampleFormat::F64,
        }
    }
}
