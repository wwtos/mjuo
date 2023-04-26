use crate::MonoSample;

use super::{pipe_player::PipePlayer, rank::Rank, sample::Pipe};
use resource_manager::{ResourceIndex, ResourceManager};

#[derive(Debug, Clone)]
struct Voice {
    player: Option<PipePlayer>,
    active: bool,
    note: u8,
}

impl Default for Voice {
    fn default() -> Self {
        Voice {
            player: None,
            active: false,
            note: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RankPlayer {
    polyphony: usize,
    voices: Vec<Voice>,
    note_to_resource_map: [Option<ResourceIndex>; 128],
}

impl RankPlayer {
    pub fn new(samples: &ResourceManager<MonoSample>, rank: &Rank, polyphony: usize) -> RankPlayer {
        let mut note_to_resource_map: [Option<ResourceIndex>; 128] = [None; 128];

        for (note, sample) in &rank.pipes {
            if let Some(resource_index) = pipes.get_index(&sample.resource.resource) {
                note_to_resource_map[sample.note as usize] = Some(resource_index);
            }
        }

        RankPlayer {
            polyphony,
            voices: Vec::with_capacity(polyphony),
            note_to_resource_map,
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

    pub fn play_note(&mut self, note: u8, samples: &ResourceManager<Pipe>) {
        if let Some(sample_index) = self.note_to_resource_map[note as usize] {
            let open_voice = self.find_open_voice(note);

            let sample = samples.borrow_resource(sample_index).unwrap();

            self.voices[open_voice].active = true;
            let voice_note = self.voices[open_voice].note;

            if let Some(player) = &mut self.voices[open_voice].player {
                if note == voice_note {
                    player.play(sample);
                } else {
                    *player = PipePlayer::new(sample);
                    player.play(sample);

                    self.voices[open_voice].note = note;
                }
            } else {
                let mut player = PipePlayer::new(sample);
                player.play(sample);

                self.voices[open_voice].player = Some(player);
                self.voices[open_voice].note = note;
            }
        }
    }

    pub fn release_note(&mut self, note: u8, samples: &ResourceManager<Pipe>) {
        for voice in &mut self.voices {
            if voice.note == note {
                if let Some(player) = &mut voice.player {
                    let sample = samples
                        .borrow_resource(self.note_to_resource_map[voice.note as usize].unwrap())
                        .unwrap();

                    player.release(sample);
                }
            }
        }
    }

    pub fn next_buffered(&mut self, buffer: &mut [f32], pipes: &ResourceManager<Pipe>) {
        for output in buffer.iter_mut() {
            *output = 0.0;
        }

        for voice in &mut self.voices {
            if voice.active {
                if let Some(pipe_index) = self.note_to_resource_map[voice.note as usize] {
                    if let Some(player) = &mut voice.player {
                        for output in buffer.iter_mut() {
                            let pipe = pipes.borrow_resource(pipe_index).unwrap();

                            *output += player.next_sample(pipe);

                            if player.is_done() {
                                voice.active = false;
                                player.reset();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn next_sample(&mut self, samples: &ResourceManager<Pipe>) -> f32 {
        let mut output_sum = 0.0;

        for voice in &mut self.voices {
            if voice.active {
                if let Some(sample_index) = self.note_to_resource_map[voice.note as usize] {
                    if let Some(player) = &mut voice.player {
                        let sample = samples.borrow_resource(sample_index).unwrap();
                        output_sum += player.next_sample(sample);

                        if player.is_done() {
                            voice.active = false;
                            player.reset();
                        }
                    }
                }
            }
        }

        output_sum
    }
}
