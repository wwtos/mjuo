use resource_manager::{ResourceIndex, ResourceManager};

use crate::SoundConfig;

use super::{rank::Rank, sample::Sample, sample_player::SamplePlayer};

#[derive(Debug, Clone)]
struct Voice {
    player: SamplePlayer,
    active: bool,
    note: u8,
}

impl Default for Voice {
    fn default() -> Self {
        Voice {
            player: SamplePlayer::new(&SoundConfig::default(), &Sample::default()),
            active: false,
            note: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RankPlayer {
    config: SoundConfig,
    polyphony: usize,
    voices: Vec<Voice>,
    note_map: [Option<ResourceIndex>; 128],
}

impl RankPlayer {
    pub fn new(config: &SoundConfig, samples: &ResourceManager<Sample>, rank: &Rank, polyphony: usize) -> RankPlayer {
        let mut note_map: [Option<ResourceIndex>; 128] = [None; 128];

        for sample in &rank.samples {
            if let Some(resource_index) = samples.get_index(&sample.resource.resource) {
                note_map[sample.note as usize] = Some(resource_index);
            }
        }

        RankPlayer {
            config: config.clone(),
            polyphony,
            voices: Vec::with_capacity(polyphony),
            note_map,
        }
    }

    fn find_open_voice(&mut self, note: u8) -> usize {
        // first, look if the voice is already active

        if let Some(existing_voice) = self.voices.iter().position(|voice| voice.note == note) {
            return existing_voice;
        }

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

    pub fn play_note(&mut self, note: u8, samples: &ResourceManager<Sample>) {
        if let Some(sample_index) = self.note_map[note as usize] {
            let open_voice = self.find_open_voice(note);

            let sample = samples.borrow_resource(sample_index).unwrap();

            self.voices[open_voice].active = true;
            self.voices[open_voice].player.init(&self.config, sample);
            self.voices[open_voice].note = note;
        }
    }

    pub fn release_note(&mut self, note: u8, samples: &ResourceManager<Sample>) {
        for voice in &mut self.voices {
            if voice.active && voice.note == note {
                let sample = samples
                    .borrow_resource(self.note_map[voice.note as usize].unwrap())
                    .unwrap();
                voice.player.release(sample);
            }
        }
    }

    pub fn next_sample(&mut self, samples: &ResourceManager<Sample>) -> f32 {
        let mut output_sum = 0.0;

        for voice in &mut self.voices {
            if voice.active {
                if let Some(sample_index) = self.note_map[voice.note as usize] {
                    let sample = samples.borrow_resource(sample_index).unwrap();
                    output_sum += voice.player.next_sample(sample);

                    if voice.player.is_done() {
                        voice.active = false;
                    }
                }
            }
        }

        output_sum
    }
}
