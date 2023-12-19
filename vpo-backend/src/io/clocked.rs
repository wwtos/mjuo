use std::{collections::BTreeSet, fmt::Debug, ops::Range};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host, HostId, SampleFormat, SupportedStreamConfigRange, SupportedStreamConfigsError,
};
use generational_arena::{Arena, Index};

#[derive(Debug, PartialEq, Eq)]
pub enum DeviceType {
    Sink,
    Source,
}
pub struct DeviceWrapper {
    pub host_id: HostId,
    pub name: String,
    pub device: Device,
    pub device_type: DeviceType,
    pub taken: bool,
}

#[derive(Debug, Clone)]
pub struct StreamConfigOptions {
    pub channels: Range<u32>,
    pub sample_rate: Range<u32>,
    pub buffer_size: Range<usize>,
    pub sample_formats: BTreeSet<MySampleFormat>,
}

pub struct DeviceManager {
    cpal_hosts: Vec<Host>,
    cpal_devices: Arena<DeviceWrapper>,
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

        let mut manager = DeviceManager {
            cpal_hosts: hosts,
            cpal_devices: Arena::new(),
        };

        manager.rescan_devices();

        manager
    }

    /// Rescans and returns a list of indexes of the new devices
    pub fn rescan_devices(&mut self) -> Vec<Index> {
        let new_devices: Vec<DeviceWrapper> = self
            .cpal_hosts
            .iter()
            .flat_map(|host| {
                let id = host.id();

                let mut sources = host
                    .input_devices()
                    .map(|devices| {
                        devices
                            .map(|device| DeviceWrapper {
                                host_id: id,
                                name: device.name().unwrap(),
                                device: device,
                                device_type: DeviceType::Source,
                                taken: false,
                            })
                            .collect()
                    })
                    .unwrap_or(vec![]);
                let sinks = host
                    .output_devices()
                    .map(|devices| {
                        devices
                            .map(|device| DeviceWrapper {
                                host_id: id,
                                name: device.name().unwrap(),
                                device: device,
                                device_type: DeviceType::Sink,
                                taken: false,
                            })
                            .collect()
                    })
                    .unwrap_or(vec![]);

                sources.extend(sinks.into_iter());

                sources
            })
            .collect();

        // this is a bit of a hack, but if it errors out when asking the device's config,
        // it's considered disconnected
        let mut to_remove = vec![];
        for (i, my_device) in self.cpal_devices.iter() {
            if !self
                .cpal_devices
                .iter()
                .any(|(_, x)| x.name == my_device.name && x.device_type == my_device.device_type)
            {
                // couldn't find `my_device in the new list, perhaps it's disconnected now?
                if let Err(SupportedStreamConfigsError::DeviceNotAvailable) = my_device.device.supported_input_configs()
                {
                    // definitely disconnected
                    to_remove.push(i);
                }
            }
        }

        for i in to_remove.into_iter().rev() {
            self.cpal_devices.remove(i);
        }

        let mut new_indexes = vec![];

        for new_device in new_devices {
            if !self
                .cpal_devices
                .iter()
                .any(|(_, x)| x.name == new_device.name && x.device_type == new_device.device_type)
            {
                println!("adding: {:?}", new_device.name);
                new_indexes.push(self.cpal_devices.insert(new_device));
            }
        }

        new_indexes
    }

    pub fn cpal_devices(&self) -> &Arena<DeviceWrapper> {
        &self.cpal_devices
    }

    pub fn cpal_input_config_options(&self, index: Index) -> Option<StreamConfigOptions> {
        self.cpal_devices[index]
            .device
            .supported_input_configs()
            .ok()
            .map(|configs| {
                let collected: Vec<_> = configs.collect();

                DeviceManager::cpal_simplify_configs(collected)
            })
    }

    pub fn cpal_output_config_options(&self, index: Index) -> Option<StreamConfigOptions> {
        self.cpal_devices[index]
            .device
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

    pub fn take_device(&mut self, index: Index) -> Option<&Device> {
        if !self.cpal_devices[index].taken {
            self.cpal_devices[index].taken = true;

            Some(&self.cpal_devices[index].device)
        } else {
            None
        }
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
