use std::{collections::BTreeMap, fs::read_to_string, path::PathBuf};

use resource_manager::ResourceId;
use serde::Deserialize;
use snafu::ResultExt;
use sound_engine::{
    sampling::rank::{Pipe, Rank},
    util::db_to_amplitude,
};

use crate::errors::{EngineError, TomlParserDeSnafu};

#[derive(Debug, Deserialize)]
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
}

fn crossfade_default() -> usize {
    256
}

#[derive(Debug, Deserialize)]
struct RankConfig {
    name: String,
    sample_location: ResourceId,
    pipe: BTreeMap<u8, RankConfigEntry>,

    // optional parameters
    #[serde(default)]
    attenuation: f32,
    #[serde(default = "crossfade_default")]
    crossfade: usize,
    #[serde(default)]
    sample_format: Option<String>,
}

const NOTE_LOOKUP: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

fn expected_sample_location(note: u8, sample_format: &str) -> String {
    format!("{:0>3}-{}.{}", note, NOTE_LOOKUP[(note % 12) as usize], sample_format)
}

/// Parses a `[rank].toml` file and converts it into a `Rank`
pub fn parse_rank(config: &str) -> Result<Rank, EngineError> {
    let parsed: RankConfig = toml::from_str(config).context(TomlParserDeSnafu)?;

    let mut pipes: BTreeMap<u8, Pipe> = BTreeMap::new();
    let sample_format = parsed.sample_format.unwrap_or("wav".into());

    for (note, entry) in parsed.pipe {
        let resource = if let Some(resource) = entry.file {
            parsed.sample_location.concat(&resource)
        } else {
            parsed
                .sample_location
                .concat(&expected_sample_location(note, &sample_format))
        };

        pipes.insert(
            note,
            Pipe {
                resource,
                note,
                cents: entry.cents,
                amplitude: db_to_amplitude(-(entry.attenuation + parsed.attenuation)),
                loop_start: entry.loop_start,
                loop_end: entry.loop_end,
                decay_index: entry.decay_index,
                release_index: entry.release_index,
                crossfade: entry.crossfade.unwrap_or(parsed.crossfade),
            },
        );
    }

    Ok(Rank {
        pipes,
        name: parsed.name,
    })
}

#[cfg(not(wasm32))]
pub fn load_rank_from_file(path: PathBuf) -> Result<Rank, EngineError> {
    use crate::errors::IoSnafu;

    let mut file = read_to_string(path).context(IoSnafu)?;

    parse_rank(&mut file)
}
