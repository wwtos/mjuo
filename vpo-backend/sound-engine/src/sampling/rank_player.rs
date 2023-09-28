use common::traits::TryRef;

use std::collections::BTreeMap;

use crate::{midi::messages::MidiData, util::interpolate::lerp, MidiBundle, MonoSample};

use super::{pipe_player::PipePlayer, rank::Rank};
use ghost_cell::{GhostCell, GhostToken};
use resource_manager::{ResourceId, ResourceManager};

#[derive(Debug, Clone)]
struct Voice {
    player: PipePlayer,
    active: bool,
    note: u8,
}

impl Default for Voice {
    fn default() -> Self {
        Voice {
            player: PipePlayer::uninitialized(),
            active: false,
            note: 255,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RankPlayer {
    polyphony: usize,
    voices: Vec<Voice>,
    note_to_sample_map: BTreeMap<u8, usize>,
    air_detune: f32,
    gain: f32,
    shelf_db_gain: f32,
    last_air_detune: f32,
    last_air_amplitude: f32,
    last_shelf_db_gain: f32,
    sample_rate: u32,
}

impl RankPlayer {
    pub fn new(rank_id: ResourceId, rank: &Rank, polyphony: usize, sample_rate: u32) -> (RankPlayer, Vec<ResourceId>) {
        let mut needed_samples: Vec<(u8, ResourceId)> = rank
            .pipes
            .iter()
            .map(|(note, sample)| (*note, sample.resource.clone()))
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
                air_detune: 1.0,
                gain: 1.0,
                shelf_db_gain: 0.0,
                last_air_detune: 1.0,
                last_air_amplitude: 1.0,
                last_shelf_db_gain: 0.0,
                sample_rate,
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

    pub fn handle_rank_updates(&mut self, rank: &Rank, samples: &ResourceManager<MonoSample>) {
        let reset_necessary = self.voices.iter().any(|voice| {
            // only check active voices to see if they have broken invariants
            if voice.active {
                if let Some(pipe) = rank.pipes.get(&voice.note) {
                    if let Some(sample) = samples.borrow_resource_by_id(&pipe.resource.resource) {
                        // reset is needed if a voice is at a position past the available buffer
                        let greatest_current_position = voice
                            .player
                            .get_position()
                            .max(voice.player.get_crossfade_position().unwrap_or(0.0))
                            .ceil() as usize;

                        greatest_current_position + 3 > sample.audio_raw.len()
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
            self.voices.push(Voice::default());

            return self.voices.len() - 1;
        }

        // if nothing else, just boot out the first entry
        // TODO: boot out longest playing note
        0
    }

    fn allocate_note<E>(&mut self, rank: &Rank, note: u8, samples: &[&impl TryRef<MonoSample, Error = E>]) {
        let pipe_and_sample = self
            .note_to_sample_map
            .get(&note)
            .and_then(|sample_index| samples[*sample_index].try_ref().ok())
            .and_then(|sample| rank.pipes.get(&note).map(|pipe| (pipe, sample)));

        if let Some((pipe, sample)) = pipe_and_sample {
            let open_voice_index = self.find_open_voice(note);
            let open_voice = &mut self.voices[open_voice_index];

            open_voice.active = true;

            if open_voice.player.is_uninitialized() {
                let mut player = PipePlayer::new(pipe, sample, self.sample_rate);

                player.set_detune(self.air_detune);
                player.set_gain(self.gain);
                player.set_shelf_db_gain(self.shelf_db_gain);

                open_voice.player = player;
                open_voice.note = note;
            } else if note == open_voice.note {
            } else {
                // TODO: don't keep reconstructing PipePlayer, it's very expensive
                // note about above TODO, I'm going to refactor the attack/release envelopes to
                // be calculated when first loading the sample, so this behavior is fine
                open_voice.player = PipePlayer::new(pipe, sample, self.sample_rate);
                open_voice.player.set_detune(self.air_detune);
                open_voice.player.set_gain(self.gain);
                open_voice.player.set_shelf_db_gain(self.shelf_db_gain);

                open_voice.note = note;
            }
        }
    }

    pub fn set_detune(&mut self, rate: f32) {
        self.air_detune = rate;
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn set_shelf_db_gain(&mut self, db_gain: f32) {
        self.shelf_db_gain = db_gain;
    }

    pub fn next_buffered<'a, 'brand, E>(
        &mut self,
        time: i64,
        midi: &GhostCell<'brand, MidiBundle>,
        rank: &Rank,
        samples: &[&impl TryRef<MonoSample, Error = E>],
        out: &[GhostCell<'brand, f32>],
        token: &mut GhostToken<'brand>,
    ) where
        E: std::fmt::Debug,
    {
        let out_len = out.len();

        for output in out.iter() {
            *output.borrow_mut(token) = 0.0;
        }

        // allocate any needed voices
        for message in midi.borrow(token) {
            match message.data {
                MidiData::NoteOn { note, .. } => {
                    self.allocate_note(rank, note, samples);
                }
                _ => {}
            }
        }

        let active_voices = self.voices.iter_mut().filter(|voice| voice.active);

        for voice in active_voices {
            let pipe_and_sample = self
                .note_to_sample_map
                .get(&voice.note)
                .and_then(|sample_index| samples[*sample_index].try_ref().ok())
                .and_then(|sample| rank.pipes.get(&voice.note).map(|pipe| (pipe, sample)));

            let mut midi_position = 0;

            if let Some((pipe, sample)) = pipe_and_sample {
                for (i, output) in out.iter().enumerate() {
                    let messages = midi.borrow(token);

                    while midi_position < messages.len() {
                        if messages[midi_position].timestamp > time + i as i64 && out_len - i > 1 {
                            break;
                        }

                        match messages[midi_position].data {
                            MidiData::NoteOn { note, .. } => {
                                if voice.note != note {
                                    midi_position += 1;
                                    continue;
                                }

                                voice.player.play(pipe, sample);
                            }
                            MidiData::NoteOff { note, .. } => {
                                if voice.note != note {
                                    midi_position += 1;
                                    continue;
                                }

                                voice.player.release(pipe, sample);
                            }
                            _ => {
                                midi_position += 1;
                                continue;
                            }
                        }

                        midi_position += 1;
                    }

                    voice
                        .player
                        .set_detune(lerp(self.last_air_detune, self.air_detune, i as f32 / out_len as f32));

                    voice
                        .player
                        .set_gain(lerp(self.last_air_amplitude, self.gain, i as f32 / out_len as f32));

                    voice.player.set_shelf_db_gain(lerp(
                        self.last_shelf_db_gain,
                        self.shelf_db_gain,
                        i as f32 / out_len as f32,
                    ));

                    *output.borrow_mut(token) += voice.player.next_sample(pipe, sample);

                    if voice.player.is_done() {
                        voice.active = false;
                        voice.player.restart();

                        break;
                    }
                }
            }
        }

        self.last_air_detune = self.air_detune;
        self.last_air_amplitude = self.gain;
        self.last_shelf_db_gain = self.shelf_db_gain;
    }
}

impl Default for RankPlayer {
    fn default() -> Self {
        RankPlayer {
            polyphony: 0,
            voices: vec![],
            note_to_sample_map: BTreeMap::new(),
            air_detune: 0.0,
            gain: 0.0,
            shelf_db_gain: 0.0,
            last_air_detune: 0.0,
            last_air_amplitude: 0.0,
            last_shelf_db_gain: 0.0,
            sample_rate: 0,
        }
    }
}
