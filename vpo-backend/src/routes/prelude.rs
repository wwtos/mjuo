use std::collections::BTreeSet;

use cpal::traits::DeviceTrait;
use log::info;
use node_engine::io_routing::{DeviceDirection, DeviceInfo, DeviceType, IoRoutes};
use node_engine::resources::Resources;
pub(super) use node_engine::state::ActionBundle;
use node_engine::state::{ActionInvalidation, GraphState};
use snafu::ResultExt;

pub(super) use crate::engine::ToAudioThread;
pub(super) use crate::errors::EngineError;
use crate::errors::NodeSnafu;
use crate::io::clocked::{DeviceManager, StreamConfigOptions};
pub(super) use crate::routes::{RouteReturn, RouteState};

pub fn state_invalidations(
    state: &mut GraphState,
    invalidations: Vec<ActionInvalidation>,
    device_manager: &mut DeviceManager,
    resources: &Resources,
) -> Result<Vec<ToAudioThread>, EngineError> {
    let mut new_engine_needed = false;
    let mut new_defaults = vec![];
    let mut updates = vec![];

    for invalidation in invalidations {
        match invalidation {
            ActionInvalidation::GraphReindexNeeded(_) => {
                new_engine_needed = true;
            }
            ActionInvalidation::NewDefaults(index, defaults) => {
                if index.graph_index == state.get_root_graph_index() {
                    new_defaults.extend(defaults.into_iter().filter_map(|(socket, value)| {
                        if let Some(value) = value.as_value() {
                            Some((index.node_index, socket, value))
                        } else {
                            None
                        }
                    }))
                }
            }
            ActionInvalidation::None => {}
            ActionInvalidation::NewNode(_) => {
                new_engine_needed = true; // TODO: be less lazy
            }
            ActionInvalidation::GraphModified(_) => {}
            ActionInvalidation::NewRouteRules { last_rules, new_rules } => {
                let last_devices: BTreeSet<(&String, DeviceDirection, DeviceType)> = last_rules
                    .devices
                    .iter()
                    .map(|rule| (&rule.name, rule.device_direction, rule.device_type))
                    .collect();
                let new_devices: BTreeSet<(&String, DeviceDirection, DeviceType)> = new_rules
                    .devices
                    .iter()
                    .map(|rule| (&rule.name, rule.device_direction, rule.device_type))
                    .collect();

                let removed = last_devices.difference(&new_devices);

                // Calculate `added` based on what devices aren't connected currently.
                // That way if an error previously occurred, it'll reattempt to
                // create the device
                let mut added: Vec<(&String, DeviceDirection, DeviceType)> = vec![];

                for rule @ (name, direction, device_type) in &new_devices {
                    match device_type {
                        DeviceType::Midi => {
                            let already_started = device_manager.midir_devices().iter().any(|(other_name, status)| {
                                *name == other_name
                                    && match direction {
                                        DeviceDirection::Sink => status.sink_handle.is_some(),
                                        DeviceDirection::Source => status.source_handle.is_some(),
                                    }
                            });

                            if !already_started {
                                added.push(rule.clone());
                            }
                        }
                        DeviceType::Stream => {
                            let already_started = device_manager.cpal_devices().iter().any(|(other_name, status)| {
                                *name == other_name
                                    && match direction {
                                        DeviceDirection::Sink => status.sink_handle.is_some(),
                                        DeviceDirection::Source => status.source_handle.is_some(),
                                    }
                            });

                            if !already_started {
                                added.push(rule.clone());
                            }
                        }
                    }
                }

                for (device, direction, device_type) in removed {
                    match direction {
                        DeviceDirection::Source => match device_type {
                            DeviceType::Midi => {
                                updates.push(ToAudioThread::RemoveMidirSource {
                                    name: device.to_string(),
                                });

                                device_manager.midir_stop_device(device, false, true);
                            }
                            DeviceType::Stream => {
                                updates.push(ToAudioThread::RemoveCpalSource {
                                    name: device.to_string(),
                                });

                                device_manager.cpal_stop_device(device, false, true);
                            }
                        },
                        DeviceDirection::Sink => match device_type {
                            DeviceType::Midi => {
                                updates.push(ToAudioThread::RemoveMidirSink {
                                    name: device.to_string(),
                                });

                                device_manager.midir_stop_device(device, true, false);
                            }
                            DeviceType::Stream => {
                                updates.push(ToAudioThread::RemoveCpalSink {
                                    name: device.to_string(),
                                });

                                device_manager.cpal_stop_device(device, true, false);
                            }
                        },
                    }
                }

                for (device, direction, device_type) in added.iter() {
                    match direction {
                        DeviceDirection::Source => match device_type {
                            DeviceType::Midi => {
                                if let Some(instance) = device_manager.midir_start_source(device) {
                                    updates.push(ToAudioThread::NewMidirSource {
                                        name: device.to_string(),
                                        source: instance,
                                    })
                                } else {
                                    return Err(EngineError::DeviceDoesNotExist {
                                        device_name: device.to_string(),
                                    });
                                }
                            }
                            DeviceType::Stream => {
                                if let Some(info) = device_manager.cpal_get_device(&device) {
                                    let options = DeviceManager::cpal_simplify_configs(
                                        info.supported_input_configs().map(|x| x.collect()).unwrap_or(vec![]),
                                    );

                                    let channels = calculate_device_channels(
                                        device,
                                        *direction,
                                        *device_type,
                                        &new_rules,
                                        options,
                                    );

                                    let device_config = new_rules.devices.iter().find(|x| &x.name == *device).unwrap();

                                    let instance = device_manager.cpal_start_source(
                                        &device,
                                        channels as u16,
                                        state.get_sound_config().sample_rate,
                                        device_config.buffer_size as u32,
                                        2,
                                    );

                                    if let Some(stream) = instance {
                                        updates.push(ToAudioThread::NewCpalSource {
                                            name: device.to_string(),
                                            source: stream,
                                        });
                                    }
                                } else {
                                    return Err(EngineError::DeviceDoesNotExist {
                                        device_name: device.to_string(),
                                    });
                                }
                            }
                        },
                        DeviceDirection::Sink => match device_type {
                            DeviceType::Midi => {
                                if let Some(instance) = device_manager.midir_start_sink(device) {
                                    updates.push(ToAudioThread::NewMidirSink {
                                        name: device.to_string(),
                                        sink: instance,
                                    })
                                } else {
                                    return Err(EngineError::DeviceDoesNotExist {
                                        device_name: device.to_string(),
                                    });
                                }
                            }
                            DeviceType::Stream => {
                                if let Some(info) = device_manager.cpal_get_device(&device) {
                                    let options = DeviceManager::cpal_simplify_configs(
                                        info.supported_output_configs().map(|x| x.collect()).unwrap_or(vec![]),
                                    );

                                    let channels = calculate_device_channels(
                                        device,
                                        *direction,
                                        *device_type,
                                        &new_rules,
                                        options,
                                    );

                                    let device_config = new_rules.devices.iter().find(|x| &x.name == *device).unwrap();

                                    let instance = device_manager.cpal_start_sink(
                                        &device,
                                        channels as u16,
                                        state.get_sound_config().sample_rate,
                                        device_config.buffer_size as u32,
                                        2,
                                    )?;

                                    updates.push(ToAudioThread::NewCpalSink {
                                        name: device.to_string(),
                                        sink: instance,
                                    });
                                } else {
                                    return Err(EngineError::DeviceDoesNotExist {
                                        device_name: device.to_string(),
                                    });
                                }
                            }
                        },
                    }

                    info!("Connected to: {}", device);
                }

                updates.push(ToAudioThread::NewRouteRules {
                    rules: new_rules.clone(),
                });
            }
        }
    }

    if new_engine_needed {
        updates.push(ToAudioThread::NewTraverser(
            state.create_traverser(resources).context(NodeSnafu)?.1,
        ));
    }

    if !new_defaults.is_empty() {
        updates.push(ToAudioThread::NewDefaults(new_defaults));
    }

    Ok(updates)
}

fn calculate_device_channels(
    device_name: &str,
    device_direction: DeviceDirection,
    device_type: DeviceType,
    rules: &IoRoutes,
    supported: StreamConfigOptions,
) -> usize {
    let requested_channels = rules
        .rules
        .iter()
        .filter_map(|rule| {
            if &rule.device_id == device_name
                && rule.device_direction == device_direction
                && rule.device_type == device_type
            {
                Some(rule.device_channel)
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);

    let actual_channels = (requested_channels + 1)
        .max(supported.channels.start as usize)
        .min(supported.channels.end as usize);

    actual_channels
}
