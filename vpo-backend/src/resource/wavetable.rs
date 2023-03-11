use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use resource_manager::{IOSnafu, LoadingError, ParserSnafu};
use rodio::{Decoder, Source};
use snafu::ResultExt;
use sound_engine::wave::wavetable::Wavetable;

fn read_wavetable(path: &Path) -> Result<Wavetable, LoadingError> {
    let mut file = File::open(path).context(IOSnafu)?;
    let mut data = String::new();
    file.read_to_string(&mut data).context(IOSnafu)?;

    serde_json::from_str(&data).context(ParserSnafu)
}

fn save_wavetable_metadata(path: &Path, metadata: &Wavetable) -> Result<(), LoadingError> {
    fs::write(path, serde_json::to_string_pretty(metadata).context(ParserSnafu)?).context(IOSnafu)
}

pub fn load_wavetable(path: PathBuf) -> Result<Wavetable, LoadingError> {
    // next, get the wavetable metadata (if it exists)
    let mut wavetable: Wavetable = Wavetable::default();
    let metadata_path = path.with_extension("json");
    if let Ok(does_exist) = path.with_extension("json").try_exists() {
        if does_exist {
            wavetable = read_wavetable(&metadata_path)?;
        } else {
            save_wavetable_metadata(&metadata_path, &wavetable)?;
        }
    }

    let file = BufReader::new(File::open(path).context(IOSnafu)?);
    let source = Decoder::new(file).unwrap();

    let sample_rate = source.sample_rate();
    let channels = source.channels();
    let length = source.size_hint().0;

    let mut buffer: Vec<f32> = Vec::with_capacity(length);
    buffer.extend(source.map(|x| x as f32 / i16::MAX as f32).step_by(channels as usize));

    wavetable.sample_rate = sample_rate;
    wavetable.wavetable = buffer;

    Ok(wavetable)
}
