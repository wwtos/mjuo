use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use regex::Regex;
use resource_manager::{IOSnafu, LoadingError, TomlParserDeSnafu, TomlParserSerSnafu};
use snafu::ResultExt;
use sound_engine::{sampling::sample::Pipe, MonoSample};
use web_sys::console;

use crate::errors::{EngineError, LoadingSnafu};

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
        let mut buffer: Vec<f32> = Vec::with_capacity(sample.len() / 4);

        for i in (0..sample.len()).step_by(4) {
            let mut frame = &sample[i..(i + 4)];
            buffer.push(frame.read_f32::<LittleEndian>().unwrap());
        }

        let sample_rate = 48_000;

        pipe.buffer = MonoSample {
            audio_raw: buffer,
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
