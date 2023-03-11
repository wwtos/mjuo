use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use regex::Regex;
use resource_manager::{IOSnafu, LoadingError, TomlParserDeSnafu, TomlParserSerSnafu};
use rodio::{Decoder, Source};
use snafu::ResultExt;
use sound_engine::{
    sampling::{envelope::calc_sample_metadata, sample::Sample},
    util::lerp,
    MonoSample,
};

fn read_sample_file(path: &Path) -> Result<Sample, LoadingError> {
    let mut file = File::open(path).context(IOSnafu)?;
    let mut data = String::new();
    file.read_to_string(&mut data).context(IOSnafu)?;

    toml::from_str(&data).context(TomlParserDeSnafu)
}

fn save_sample_metadata(path: &Path, metadata: &Sample) -> Result<(), LoadingError> {
    fs::write(path, toml::to_string(metadata).context(TomlParserSerSnafu)?).context(IOSnafu)
}

pub fn load_sample(path: PathBuf) -> Result<Sample, LoadingError> {
    println!("loading: {:?}", path);
    let file = BufReader::new(File::open(&path).context(IOSnafu)?);
    let source = Decoder::new(file).unwrap();

    let sample_rate = source.sample_rate();
    let channels = source.channels();
    let length = source.size_hint().0;

    let mut buffer: Vec<f32> = Vec::with_capacity(length);
    buffer.extend(source.map(|x| x as f32 / i16::MAX as f32).step_by(channels as usize));

    // next, get the sample metadata (if it exists)
    let mut sample: Sample = Sample::default();
    let metadata_path = path.with_extension("toml");
    if let Ok(does_exist) = path.with_extension("toml").try_exists() {
        if does_exist {
            sample = read_sample_file(&metadata_path)?;
        } else {
            let note_number = check_for_note_number(&path.file_stem().unwrap().to_string_lossy());
            let possible_freq = note_number.map(|note| 440.0 * 2_f64.powf((note as i16 - 69) as f64 / 12.0));

            let metadata = calc_sample_metadata(&buffer, sample_rate, possible_freq);

            sample.decay_index = metadata.decay_index;
            sample.sustain_index = metadata.sustain_index;
            sample.release_index = metadata.release_index;
            sample.loop_start = metadata.loop_start;
            sample.loop_end = metadata.loop_end;
            sample.note = metadata.note;
            sample.cents = metadata.cents;

            save_sample_metadata(&metadata_path, &sample)?;
        }
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
