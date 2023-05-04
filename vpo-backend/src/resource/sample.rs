use std::path::Path;

use regex::Regex;
use sound_engine::MonoSample;

use super::decode_audio::decode_audio;
use crate::errors::EngineError;

#[cfg(not(wasm32))]
pub fn load_sample(location: &Path) -> Result<MonoSample, EngineError> {
    use crate::{errors::FileSnafu, resource::util::first_channel_only};
    use snafu::ResultExt;
    use std::fs::File;
    use symphonia::core::probe::Hint;

    let file = Box::new(File::open(location).context(FileSnafu)?);
    let mut hint = Hint::new();

    if let Some(extension) = location.extension() {
        hint.with_extension(&extension.to_string_lossy());
    }

    let (audio, spec) = decode_audio(file, hint)?;

    Ok(MonoSample {
        audio_raw: first_channel_only(&audio, spec.channels.count()),
        sample_rate: spec.rate,
    })
}

pub fn check_for_note_number(file_prefix: &str) -> Option<u8> {
    let get_numbers = Regex::new(r"([0-9]+)").unwrap();
    let remove_leading_zeroes = Regex::new(r"^0+").unwrap();

    get_numbers
        .captures(file_prefix)
        .and_then(|captures| captures.get(0))
        .map(|numbers| remove_leading_zeroes.replace(numbers.as_str(), ""))
        .and_then(|numbers_trimmed| numbers_trimmed.parse().ok())
}
