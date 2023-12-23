use std::{collections::BTreeSet, fmt::Debug, ops::Range};

use clocked::{
    cpal::{start_cpal_sink, start_cpal_source, CpalSink, CpalSource},
    midir::{start_midir_sink, start_midir_source, MidirSink, MidirSource},
};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host, HostId, SampleFormat, SampleRate, Stream, StreamConfig, SupportedStreamConfigRange,
};
use generational_arena::{Arena, Index};
use midir::{MidiInput, MidiInputPort, MidiOutput, MidiOutputPort};
use node_engine::io_routing::DeviceDirection;

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

#[derive(Debug)]
pub struct CpalDeviceStatus {
    pub host_id: HostId,
    pub name: String,
    pub is_sink: bool,
    pub is_source: bool,
    pub sink_taken: bool,
    pub source_taken: bool,
}

pub struct MidirDeviceStatus {
    pub name: String,
    pub device: MidiDevice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CpalIndex(pub Index);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MidirIndex(pub Index);

#[derive(Debug, Clone)]
pub struct StreamConfigOptions {
    pub channels: Range<u32>,
    pub sample_rate: Range<u32>,
    pub buffer_size: Range<usize>,
    pub sample_formats: BTreeSet<MySampleFormat>,
}

pub struct DeviceManager {
    cpal_hosts: Vec<Host>,
    cpal_statuses: Arena<CpalDeviceStatus>,
    midir_devices: Arena<MidirDeviceStatus>,
    midir_input_scan: MidiInput,
    midir_output_scan: MidiOutput,
}

pub struct ScanResult {
    pub cpal_added: Vec<CpalIndex>,
    pub cpal_removed: Vec<CpalIndex>,
    pub midir_added: Vec<MidirIndex>,
    pub midir_removed: Vec<MidirIndex>,
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
            cpal_statuses: Arena::new(),
            midir_devices: Arena::new(),
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

    fn rescan_midir_devices(&mut self) -> (Vec<MidirIndex>, Vec<MidirIndex>) {
        let mut midi_in = MidiInput::new("midi scan").unwrap();
        midi_in.ignore(midir::Ignore::None);
        let midi_out = MidiOutput::new("midi scan").unwrap();

        let mut current_sources: Vec<MidirDeviceStatus> = midi_in
            .ports()
            .into_iter()
            .map(|port| MidirDeviceStatus {
                name: midi_in.port_name(&port).unwrap(),
                device: MidiDevice::Source(port),
            })
            .collect();
        let current_sinks: Vec<MidirDeviceStatus> = midi_out
            .ports()
            .into_iter()
            .map(|port| MidirDeviceStatus {
                name: midi_out.port_name(&port).unwrap(),
                device: MidiDevice::Sink(port),
            })
            .collect();

        current_sources.extend(current_sinks.into_iter());
        let current_devices = current_sources;

        // this is a bit of a hack, but if it errors out when asking the device's config,
        // it's considered disconnected
        let mut to_remove = vec![];
        for (i, my_device) in self.midir_devices.iter() {
            if !self
                .midir_devices
                .iter()
                .any(|(_, x)| x.name == my_device.name && x.device.device_type() == my_device.device.device_type())
            {
                // couldn't find `my_device` in the new list, perhaps it's disconnected now?
                match &my_device.device {
                    MidiDevice::Source(source) => {
                        if self.midir_input_scan.port_name(&source).is_err() {
                            // definitely disconnected
                            to_remove.push(MidirIndex(i));
                        }
                    }
                    MidiDevice::Sink(sink) => {
                        if self.midir_output_scan.port_name(&sink).is_err() {
                            // definitely disconnected
                            to_remove.push(MidirIndex(i));
                        }
                    }
                }
            }
        }

        for i in to_remove.iter() {
            self.cpal_statuses.remove(i.0);
        }

        let mut new_indexes = vec![];

        for new_device in current_devices {
            if !self
                .midir_devices
                .iter()
                .any(|(_, x)| x.name == new_device.name && x.device.device_type() == new_device.device.device_type())
            {
                println!("adding: {:?}", new_device.name);
                new_indexes.push(MidirIndex(self.midir_devices.insert(new_device)));
            }
        }

        (new_indexes, to_remove)
    }

    fn rescan_cpal_devices(&mut self) -> (Vec<CpalIndex>, Vec<CpalIndex>) {
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

                    CpalDeviceStatus {
                        host_id: id,
                        name: device.name().unwrap(),
                        is_sink,
                        is_source,
                        sink_taken: false,
                        source_taken: false,
                    }
                });

                devices
            })
            .collect();

        let mut new_indexes = vec![];

        for new_device in current_device_list {
            if !self.cpal_statuses.iter().any(|(_, x)| {
                x.name == new_device.name && x.is_sink == new_device.is_sink && x.is_source == new_device.is_source
            }) {
                println!("adding: {:?}", new_device.name);
                new_indexes.push(CpalIndex(self.cpal_statuses.insert(new_device)));
            }
        }

        (new_indexes, vec![])
    }

    pub fn cpal_devices(&self) -> &Arena<CpalDeviceStatus> {
        &self.cpal_statuses
    }

    pub fn cpal_status_by_name(&self, name: &str, device_dir: DeviceDirection) -> Option<CpalIndex> {
        self.cpal_statuses
            .iter()
            .find(|(_, device)| {
                &device.name == name
                    && match device_dir {
                        DeviceDirection::Sink => device.is_sink,
                        DeviceDirection::Source => device.is_source,
                    }
            })
            .map(|x| CpalIndex(x.0))
    }

    pub fn cpal_get_device(&self, device: CpalIndex) -> Option<Device> {
        let status = self.cpal_statuses.get(device.0)?;
        let host = cpal::host_from_id(status.host_id).ok()?;

        host.devices()
            .ok()?
            .find(|x| x.name().unwrap_or_default() == status.name)
    }

    pub fn cpal_input_config_options(&self, index: CpalIndex) -> Option<StreamConfigOptions> {
        self.cpal_get_device(index)?
            .supported_input_configs()
            .ok()
            .map(|configs| {
                let collected: Vec<_> = configs.collect();

                DeviceManager::cpal_simplify_configs(collected)
            })
    }

    pub fn cpal_output_config_options(&self, index: CpalIndex) -> Option<StreamConfigOptions> {
        self.cpal_get_device(index)?
            .supported_output_configs()
            .ok()
            .map(|configs| {
                let collected: Vec<_> = configs.collect();

                DeviceManager::cpal_simplify_configs(collected)
            })
    }

    pub fn cpal_simplify_configs(configs: Vec<SupportedStreamConfigRange>) -> StreamConfigOptions {
        let min_channels = configs.iter().map(|x| x.channels()).min().unwrap_or(0) as u32;
        let mut max_channels = configs.iter().map(|x| x.channels()).max().unwrap_or(0) as u32;

        if max_channels == 32 {
            max_channels = u32::MAX; // silly cpal
        }

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
        index: CpalIndex,
        channels: u16,
        sample_rate: u32,
        buffer_size: u32,
        periods: usize,
    ) -> Option<(Stream, CpalSink)> {
        if let Some(device) = self.cpal_statuses.get_mut(index.0) {
            if device.sink_taken {
                return None;
            }

            device.sink_taken = true;

            start_cpal_sink(
                &self.cpal_get_device(index)?,
                &StreamConfig {
                    channels: channels as u16,
                    sample_rate: SampleRate(sample_rate),
                    buffer_size: cpal::BufferSize::Fixed(buffer_size),
                },
                SampleFormat::F32,
                buffer_size as usize,
                periods,
            )
            .ok()
        } else {
            None
        }
    }

    pub fn cpal_start_source(
        &mut self,
        index: CpalIndex,
        channels: u16,
        sample_rate: u32,
        buffer_size: u32,
        periods: usize,
    ) -> Option<(Stream, CpalSource)> {
        if let Some(device) = self.cpal_statuses.get_mut(index.0) {
            if device.source_taken {
                return None;
            }

            device.source_taken = true;

            start_cpal_source(
                &self.cpal_get_device(index)?,
                &StreamConfig {
                    channels: channels as u16,
                    sample_rate: SampleRate(sample_rate),
                    buffer_size: cpal::BufferSize::Fixed(buffer_size),
                },
                SampleFormat::F32,
                buffer_size as usize,
                periods,
            )
            .ok()
        } else {
            None
        }
    }

    pub fn midir_devices(&self) -> &Arena<MidirDeviceStatus> {
        &self.midir_devices
    }

    pub fn midir_start_sink(&mut self, sink_name: String, port_name: String) -> Option<MidirSink> {
        let sink = MidiOutput::new(&sink_name).ok()?;

        let ports = sink.ports();
        let port = ports
            .iter()
            .find(|port| sink.port_name(port).map(|x| x == port_name).unwrap_or(false))?;

        start_midir_sink(sink, port, &sink_name).ok()
    }

    pub fn midir_start_source(&mut self, sink_name: String, port_name: String) -> Option<MidirSource> {
        let source = MidiInput::new(&sink_name).ok()?;

        let ports = source.ports();
        let port = ports
            .iter()
            .find(|port| source.port_name(port).map(|x| x == port_name).unwrap_or(false))?;

        start_midir_source(source, port, &sink_name).ok()
    }
}

pub struct DeviceState {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
