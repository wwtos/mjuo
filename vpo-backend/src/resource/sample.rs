use std::{
    fs::{self, File},
    io::{Cursor, Read},
    path::Path,
};

use regex::Regex;
use resource_manager::{IOSnafu, LoadingError, TomlParserDeSnafu, TomlParserSerSnafu};
use snafu::ResultExt;
use sound_engine::{sampling::sample::Pipe, MonoSample};
use symphonia::core::probe::Hint;
use web_sys::console;

use crate::errors::{EngineError, LoadingSnafu};

use super::util::{decode_audio, mix_to_mono};

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

pub fn load_pipe(config: String, resource: Option<Vec<u8>>) -> Result<Pipe, EngineError> {
    let mut pipe = parse_pipe_config(&config).context(LoadingSnafu)?;

    if let Some(sample) = resource {
        let (buffer, spec) = decode_audio(Box::new(Cursor::new(sample)), Hint::new())?;

        let sample_rate = spec.rate;
        let channels = spec.channels.count();

        console::log_1(
            &format!(
                "Buffer len: {}, channels: {}, sample_rate: {}",
                buffer.len(),
                channels,
                sample_rate
            )
            .into(),
        );
        let buffer_mono = mix_to_mono(&buffer, channels);

        pipe.buffer = MonoSample {
            audio_raw: buffer_mono,
            sample_rate,
        };
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
