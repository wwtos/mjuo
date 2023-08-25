use std::{collections::BTreeMap, fs::read_to_string, path::Path};

use resource_manager::{ResourceId, ResourceManager};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use sound_engine::{
    sampling::{
        phase_calculator::PhaseCalculator,
        pipe_player::{envelope_indexes, EnvelopeType},
        rank::{Pipe, Rank},
    },
    util::db_to_gain,
    MonoSample,
};

use crate::errors::{EngineError, TomlParserDeSnafu};

#[derive(Debug, Serialize, Deserialize)]
struct RankConfigEntry {
    cents: i16,
    decay_index: usize,
    loop_start: usize,
    loop_end: usize,
    release_index: usize,

    // optional parameters
    #[serde(default)]
    crossfade: Option<usize>,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    attenuation: f32,
    #[serde(default)]
    even_harm_atten: f32,
}

fn crossfade_default() -> usize {
    256
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RankConfig {
    name: String,
    sample_location: ResourceId,
    pipe: BTreeMap<String, RankConfigEntry>,

    // optional parameters
    #[serde(default)]
    attenuation: f32,
    #[serde(default = "crossfade_default")]
    crossfade: usize,
    #[serde(default)]
    sample_format: Option<String>,
}

impl RankConfig {
    pub fn from_rank(rank: Rank, sample_location: ResourceId) -> Self {
        let entries: BTreeMap<String, RankConfigEntry> = rank
            .pipes
            .into_iter()
            .map(|(note, pipe)| {
                (
                    note.to_string(),
                    RankConfigEntry {
                        cents: 0,
                        decay_index: pipe.decay_index,
                        loop_start: pipe.loop_start,
                        loop_end: pipe.loop_end,
                        release_index: pipe.release_index,
                        crossfade: Some(pipe.crossfade),
                        file: None,
                        attenuation: 0.0,
                        even_harm_atten: 0.0,
                    },
                )
            })
            .collect();

        RankConfig {
            name: rank.name,
            sample_location,
            pipe: entries,
            attenuation: 0.0,
            crossfade: 0,
            sample_format: None,
        }
    }
}

const NOTE_LOOKUP: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

pub fn expected_sample_location(note: u8, sample_format: &str) -> String {
    format!("{:0>3}-{}.{}", note, NOTE_LOOKUP[(note % 12) as usize], sample_format)
}

/// Parses a `[rank].toml` file and converts it into a `Rank`
pub fn parse_rank(config: &str, samples: &ResourceManager<MonoSample>) -> Result<Rank, EngineError> {
    let parsed: RankConfig = toml_edit::de::from_str(config).context(TomlParserDeSnafu)?;

    let mut pipes: BTreeMap<u8, Pipe> = BTreeMap::new();
    let sample_format = parsed.sample_format.unwrap_or("wav".into());

    for (note, entry) in parsed.pipe {
        let note: u8 = match note.parse() {
            Ok(note) => note,
            Err(_) => continue,
        };

        let resource = if let Some(resource) = entry.file {
            parsed.sample_location.concat(&resource)
        } else {
            parsed
                .sample_location
                .concat(&expected_sample_location(note, &sample_format))
        };

        if let Some(sample) = samples
            .get_index(&resource.resource)
            .and_then(|x| samples.borrow_resource(x))
        {
            let buffer_rate = sample.sample_rate;
            let freq = (440.0 / 32.0) * 2_f32.powf((note - 9) as f32 / 12.0 + (entry.cents as f32 / 1200.0));
            let amp_window_size = (buffer_rate as f32 / freq) as usize * 2;

            let phase_calculator = PhaseCalculator::new(freq, buffer_rate);

            let attack_envelope = envelope_indexes(
                entry.decay_index,
                entry.release_index,
                sample,
                amp_window_size,
                EnvelopeType::Attack,
            );
            let release_envelope = envelope_indexes(
                entry.decay_index,
                entry.release_index,
                sample,
                amp_window_size,
                EnvelopeType::Release,
            );

            pipes.insert(
                note,
                Pipe {
                    resource,
                    freq,
                    amplitude: db_to_gain(-(entry.attenuation + parsed.attenuation)),
                    loop_start: entry.loop_start,
                    loop_end: entry.loop_end,
                    decay_index: entry.decay_index,
                    release_index: entry.release_index,
                    crossfade: entry.crossfade.unwrap_or(parsed.crossfade),
                    comb_coeff: db_to_gain(entry.even_harm_atten),
                    amp_window_size,
                    phase_calculator,
                    attack_envelope: attack_envelope,
                    release_envelope: release_envelope,
                },
            );
        }
    }

    Ok(Rank {
        pipes,
        name: parsed.name,
    })
}

#[cfg(any(unix, windows))]
pub fn load_rank_from_file(path: &Path, samples: &ResourceManager<MonoSample>) -> Result<Rank, EngineError> {
    use crate::errors::IoSnafu;

    let file = read_to_string(path).context(IoSnafu)?;

    parse_rank(&file, samples)
}
