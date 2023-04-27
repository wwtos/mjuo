use std::collections::BTreeMap;

use crate::MonoSample;

use super::{pipe_player::PipePlayer, rank::Rank};
use resource_manager::{ResourceIndex, ResourceManager};

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
            note: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RankPlayer {
    polyphony: usize,
    voices: Vec<Voice>,
    note_to_sample_map: BTreeMap<u8, ResourceIndex>,
}

impl RankPlayer {
    pub fn new(samples: &ResourceManager<MonoSample>, rank: &Rank, polyphony: usize) -> RankPlayer {
        let mut note_to_sample_map: BTreeMap<u8, ResourceIndex> = BTreeMap::new();

        for (note, sample) in &rank.pipes {
            if let Some(resource_index) = samples.get_index(&sample.resource.resource) {
                note_to_sample_map.insert(sample.note, resource_index);
            }
        }

        RankPlayer {
            polyphony,
            voices: Vec::with_capacity(polyphony),
            note_to_sample_map,
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

    pub fn play_note(&mut self, note: u8, rank: &Rank, samples: &ResourceManager<MonoSample>) {
        let pipe_and_sample = self
            .note_to_sample_map
            .get(&note)
            .and_then(|sample_index| samples.borrow_resource(*sample_index))
            .and_then(|sample| rank.pipes.get(&note).map(|pipe| (pipe, sample)));

        if let Some((pipe, sample)) = pipe_and_sample {
            let open_voice_index = self.find_open_voice(note);
            let open_voice = &mut self.voices[open_voice_index];

            open_voice.active = true;

            if open_voice.player.is_uninitialized() {
                let mut player = PipePlayer::new(pipe, sample);
                player.play(pipe, sample);

                open_voice.player = player;
                open_voice.note = note;
            } else {
                if note == open_voice.note {
                    open_voice.player.play(pipe, sample);
                } else {
                    // TODO: don't keep reconstructing PipePlayer, it's very expensive
                    open_voice.player = PipePlayer::new(pipe, sample);
                    open_voice.player.play(pipe, sample);

                    open_voice.note = note;
                }
            }
        }
    }

    pub fn release_note(&mut self, note: u8, rank: &Rank, samples: &ResourceManager<MonoSample>) {
        if let Some(voice) = self.voices.iter_mut().find(|voice| voice.note == note) {
            let pipe_and_sample = self
                .note_to_sample_map
                .get(&note)
                .and_then(|sample_index| samples.borrow_resource(*sample_index))
                .and_then(|sample| rank.pipes.get(&note).map(|pipe| (pipe, sample)));

            if let Some((pipe, sample)) = pipe_and_sample {
                voice.player.release(pipe, sample);
            }
        }
    }

    pub fn next_buffered(&mut self, rank: &Rank, samples: &ResourceManager<MonoSample>, out: &mut [f32]) {
        for output in out.iter_mut() {
            *output = 0.0;
        }

        let active_voices = self.voices.iter_mut().filter(|voice| voice.active);

        for voice in active_voices {
            let pipe_and_sample = self
                .note_to_sample_map
                .get(&voice.note)
                .and_then(|sample_index| samples.borrow_resource(*sample_index))
                .and_then(|sample| rank.pipes.get(&voice.note).map(|pipe| (pipe, sample)));

            if let Some((pipe, sample)) = pipe_and_sample {
                for output in out.iter_mut() {
                    *output += voice.player.next_sample(pipe, sample);

                    if voice.player.is_done() {
                        voice.active = false;
                        voice.player.reset();
                    }
                }
            }
        }
    }
}
