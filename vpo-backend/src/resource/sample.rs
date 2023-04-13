use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use regex::Regex;
use resource_manager::{IOSnafu, LoadingError, TomlParserDeSnafu, TomlParserSerSnafu};
use snafu::ResultExt;
use sound_engine::{sampling::sample::Pipe, util::lerp, MonoSample};
use symphonia::core::{io::MediaSource, probe::Hint};

use crate::errors::{EngineError, LoadingSnafu};

use super::util::decode_audio;

fn parse_pipe_config(contents: &str) -> Result<Pipe, LoadingError> {
    toml::from_str(contents).context(TomlParserDeSnafu)
}

pub fn read_pipe_config(path: &Path) -> Result<Pipe, LoadingError> {
    let mut file = File::open(path).context(IOSnafu)?;
    let mut data = String::new();
    file.read_to_string(&mut data).context(IOSnafu)?;

    parse_pipe_config(&data)
}

fn save_sample_metadata(path: &Path, metadata: &Pipe) -> Result<(), LoadingError> {
    fs::write(path, toml::to_string(metadata).context(TomlParserSerSnafu)?).context(IOSnafu)
}

pub fn load_pipe(config: String, resource: Option<Box<dyn MediaSource>>) -> Result<Pipe, EngineError> {
    let mut pipe = parse_pipe_config(&config).context(LoadingSnafu)?;

    if let Some(sample) = resource {
        let (buffer, spec) = decode_audio(sample, Hint::new())?;

        let sample_rate = spec.rate;
        let channels = spec.channels.count();

        pipe.buffer = MonoSample {
            audio_raw: buffer,
            sample_rate,
        };

        if pipe.crossfade > 0 {
            // calculate crossfade here
            let mut crossfade_audio: Vec<f32> = Vec::new();

            let audio = &pipe.buffer.audio_raw;

            for i in 0..pipe.crossfade {
                crossfade_audio.push(lerp(
                    audio[pipe.loop_end + i],
                    audio[pipe.loop_start + i],
                    i as f32 / pipe.crossfade as f32,
                ));
            }
        }
    }

    Ok(pipe)
}

fn check_for_note_number(file_prefix: &str) -> Option<u8> {
    let get_numbers = Regex::new(r"([0-9]+)").unwrap();
    let remove_leading_zeroes = Regex::new(r"^0+").unwrap();

    get_numbers
        .captures(file_prefix)
        .and_then(|captures| captures.get(0))
        .map(|numbers| remove_leading_zeroes.replace(numbers.as_str(), ""))
        .and_then(|numbers_trimmed| numbers_trimmed.parse().ok())
}
