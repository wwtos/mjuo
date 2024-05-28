use common::osc::OscView;
use common::osc_midi::{NOTE_OFF, NOTE_ON};
use common::read_osc;
use common::traits::TryRef;

use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::{MonoSample, SoundConfig};

use super::{rank::Rank, Resource, Voice};
use common::resource_manager::{ResourceId, ResourceManager};

#[derive(Debug, Clone)]
struct VoiceInfo<V: Voice> {
    player: V,
    active: bool,
    note: u8,
}

impl<V: Voice> Default for VoiceInfo<V> {
    fn default() -> Self {
        VoiceInfo {
            player: V::default(),
            active: false,
            note: 255,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RankPlayer<V: Voice> {
    polyphony: usize,
    voices: Vec<VoiceInfo<V>>,
    note_to_sample_map: BTreeMap<u8, usize>,
    param: V::Param,
    sound_config: SoundConfig,
}

impl<V: Voice> RankPlayer<V> {
    pub fn new(
        rank_id: ResourceId,
        rank: &Rank<V::Resource>,
        polyphony: usize,
        sound_config: SoundConfig,
    ) -> (RankPlayer<V>, Vec<ResourceId>) {
        let mut needed_samples: Vec<(u8, ResourceId)> = rank
            .notes
            .iter()
            .map(|(note, resource)| (*note, resource.resource_id().clone()))
            .collect();

        needed_samples.sort_by_key(|x| x.0);

        let note_to_sample_map: BTreeMap<u8, usize> = needed_samples
            .iter()
            .enumerate()
            .map(|(i, (note, _))| (*note, i))
            .collect();

        let mut resource_list: Vec<ResourceId> = vec![rank_id];
        resource_list.extend(needed_samples.into_iter().map(|(_, resource)| resource));

        (
            RankPlayer {
                polyphony,
                voices: Vec::with_capacity(polyphony),
                note_to_sample_map,
                param: V::Param::default(),
                sound_config,
            },
            resource_list,
        )
    }

    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.active = false;
            voice.note = 255;
        }
    }

    pub fn handle_rank_updates(&mut self, rank: &Rank<V::Resource>, samples: &ResourceManager<MonoSample>) {
        let reset_necessary = self.voices.iter().any(|voice| {
            // only check active voices to see if they have broken invariants
            if voice.active {
                if let Some(resource) = rank.notes.get(&voice.note) {
                    if samples
                        .borrow_resource_by_id(&resource.resource_id().resource)
                        .is_some()
                    {
                        // the resource still exists
                        false
                    } else {
                        // reset is needed if a sample was removed
                        true
                    }
                } else {
                    // reset is needed if a voice was removed from the rank config
                    true
                }
            } else {
                false
            }
        });

        if reset_necessary {
            self.reset();
        }
    }

    fn find_open_voice(&mut self, note: u8) -> usize {
        // first, look if the voice is already active
        if let Some(existing_voice) = self.voices.iter().position(|voice| voice.note == note) {
            return existing_voice;
        }

        // is there a voice that is no longer active?
        if let Some(inactive_voice) = self.voices.iter().position(|voice| !voice.active) {
            return inactive_voice;
        };

        // else, see if we're at full capacity yet
        if self.voices.len() < self.polyphony {
            self.voices.push(VoiceInfo::default());

            return self.voices.len() - 1;
        }

        // if nothing else, just boot out the first entry
        // TODO: boot out longest playing note
        0
    }

    fn allocate_note<E>(&mut self, rank: &Rank<V::Resource>, note: u8, samples: &[impl TryRef<V::Sample, Error = E>]) {
        let resource_and_sample = self
            .note_to_sample_map
            .get(&note)
            .and_then(|sample_index| samples[*sample_index].try_ref().ok())
            .and_then(|sample| rank.notes.get(&note).map(|pipe| (pipe, sample)));

        if let Some((pipe, sample)) = resource_and_sample {
            let open_voice_index = self.find_open_voice(note);
            let open_voice = &mut self.voices[open_voice_index];

            open_voice.active = true;

            if !open_voice.player.active() {
                let mut player = V::new(pipe, sample, self.sound_config.clone());
                player.set_param(&self.param);

                open_voice.player = player;
                open_voice.note = note;
            } else if note == open_voice.note {
                // nothing to do
            } else {
                open_voice.player = V::new(pipe, sample, self.sound_config.clone());
                open_voice.player.set_param(&self.param);

                open_voice.note = note;
            }
        }
    }

    pub fn set_param(&mut self, param: V::Param) {
        self.param = param;
    }

    pub fn next_buffered<'a, E>(
        &mut self,
        osc: OscView,
        rank: &Rank<V::Resource>,
        samples: &[impl TryRef<V::Sample, Error = E>],
        out: &mut [f32],
    ) where
        E: std::fmt::Debug,
    {
        for output in out.iter_mut() {
            *output = 0.0;
        }

        // allocate any needed voices
        osc.all_messages(|_, _, message| match message.address().to_str() {
            Ok(NOTE_ON) => {
                if let Some((channel, note, velocity)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) {
                    self.allocate_note(rank, note as u8, samples);
                }
            }
            _ => {}
        });

        let active_voices = self.voices.iter_mut().filter(|voice| voice.active);

        for voice in active_voices {
            let pipe_and_sample = self
                .note_to_sample_map
                .get(&voice.note)
                .and_then(|sample_index| samples[*sample_index].try_ref().ok())
                .and_then(|sample| rank.notes.get(&voice.note).map(|pipe| (pipe, sample)));

            let Some((pipe, sample)) = pipe_and_sample else {
                continue;
            };

            osc.all_messages(|_, _, message| match message.address().to_str() {
                Ok(NOTE_ON) => {
                    dbg!("note on");

                    if let Some((_, note, _)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) {
                        dbg!("note on: {}", note);

                        if voice.note == note as u8 {
                            voice.player.attack(pipe, sample);
                        }
                    }
                }
                Ok(NOTE_OFF) => {
                    if let Some((_, note, _)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) {
                        if voice.note == note as u8 {
                            voice.player.release(pipe, sample);
                        }
                    }
                }
                _ => {}
            });

            voice.player.set_param(&self.param);

            for output in out.iter_mut() {
                *output += voice.player.step(pipe, sample);

                if !voice.player.active() {
                    voice.active = false;
                    voice.player.reset();

                    break;
                }
            }
        }
    }
}

impl<V: Voice> Default for RankPlayer<V> {
    fn default() -> Self {
        RankPlayer {
            polyphony: 0,
            voices: vec![],
            note_to_sample_map: BTreeMap::new(),
            sound_config: SoundConfig::default(),
            param: V::Param::default(),
        }
    }
}
