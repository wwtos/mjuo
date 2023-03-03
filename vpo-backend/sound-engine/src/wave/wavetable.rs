use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::Path,
};

use resource_manager::{IOSnafu, LoadingError, ParserSnafu, Resource};
use rodio::{Decoder, Source};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wavetable {
    #[serde(skip)]
    pub sample_rate: u32,
    #[serde(skip)]
    pub wavetable: Vec<f32>,
}

impl Default for Wavetable {
    fn default() -> Self {
        Self {
            sample_rate: 1,
            wavetable: Vec::new(),
        }
    }
}

fn load_wavetable(path: &Path) -> Result<Wavetable, LoadingError> {
    let mut file = File::open(path).context(IOSnafu)?;
    let mut data = String::new();
    file.read_to_string(&mut data).context(IOSnafu)?;

    serde_json::from_str(&data).context(ParserSnafu)
}

fn save_wavetable_metadata(path: &Path, metadata: &Wavetable) -> Result<(), LoadingError> {
    fs::write(path, serde_json::to_string_pretty(metadata).context(ParserSnafu)?).context(IOSnafu)
}

impl Resource for Wavetable {
    fn load_resource(path: &Path) -> Result<Self, LoadingError>
    where
        Self: Sized,
    {
        // next, get the wavetable metadata (if it exists)
        let mut wavetable: Wavetable = Wavetable::default();
        let metadata_path = path.with_extension("json");
        if let Ok(does_exist) = path.with_extension("json").try_exists() {
            if does_exist {
                wavetable = load_wavetable(&metadata_path)?;
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
}
