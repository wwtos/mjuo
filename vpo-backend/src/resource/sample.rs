use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use regex::Regex;
use resource_manager::{IOSnafu, LoadingError, TomlParserDeSnafu, TomlParserSerSnafu};
use snafu::ResultExt;
use sound_engine::{
    sampling::{envelope::calc_sample_metadata, sample::Sample},
    util::lerp,
    MonoSample,
};
use symphonia::core::{io::MediaSource, probe::Hint};

use crate::errors::{EngineError, LoadingSnafu};

use super::util::decode_audio;

fn parse_sample_file(contents: &str) -> Result<Sample, LoadingError> {
    toml::from_str(contents).context(TomlParserDeSnafu)
}

fn read_sample_file(path: &Path) -> Result<Sample, LoadingError> {
    let mut file = File::open(path).context(IOSnafu)?;
    let mut data = String::new();
    file.read_to_string(&mut data).context(IOSnafu)?;

    parse_sample_file(&data)
}

fn save_sample_metadata(path: &Path, metadata: &Sample) -> Result<(), LoadingError> {
    fs::write(path, toml::to_string(metadata).context(TomlParserSerSnafu)?).context(IOSnafu)
}

pub fn load_sample(resource: Box<dyn MediaSource>, config: Option<String>) -> Result<Sample, EngineError> {
    let (buffer, spec) = decode_audio(resource, Hint::new())?;

    let sample_rate = spec.rate;
    let channels = spec.channels.count();

    let mut sample = Sample::default();

    // next, get the sample metadata (if it exists)
    if let Some(associated_resource) = config {
        sample = parse_sample_file(&associated_resource).context(LoadingSnafu)?;
    } else {
        let metadata = calc_sample_metadata(&buffer, sample_rate, None);

        sample.decay_index = metadata.decay_index;
        sample.sustain_index = metadata.sustain_index;
        sample.release_index = metadata.release_index;
        sample.loop_start = metadata.loop_start;
        sample.loop_end = metadata.loop_end;
        sample.note = metadata.note;
        sample.cents = metadata.cents;
    }

    sample.buffer = MonoSample {
        audio_raw: buffer,
        sample_rate,
    };

    if sample.crossfade > 0 {
        // calculate crossfade here
        let mut crossfade_audio: Vec<f32> = Vec::new();

        let audio = &sample.buffer.audio_raw;

        for i in 0..sample.crossfade {
            crossfade_audio.push(lerp(
                audio[sample.loop_end + i],
                audio[sample.loop_start + i],
                i as f32 / sample.crossfade as f32,
            ));
        }

        sample.crossfade_buffer = MonoSample {
            audio_raw: crossfade_audio,
            sample_rate,
        }
    }

    Ok(sample)
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
