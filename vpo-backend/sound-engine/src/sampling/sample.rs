use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::Path,
};

use resource_manager::{IOSnafu, LoadingError, ParserSnafu, Resource};
use rodio::{Decoder, Source};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{midi::messages::Note, MonoSample};

use super::interpolate::lerp;

#[derive(Serialize, Deserialize, Debug)]
pub struct Sample {
    pub loop_start: Option<usize>,
    pub loop_end: Option<usize>,
    pub release: Option<usize>,
    pub crossfade: usize,
    pub note: Option<Note>,
    pub cents: Option<i16>,
    #[serde(skip)]
    pub buffer: MonoSample,
    #[serde(skip)]
    pub crossfade_buffer: MonoSample,
}

impl Default for Sample {
    fn default() -> Self {
        Self {
            loop_start: Some(0),
            loop_end: Some(100),
            release: Some(100),
            crossfade: 0,
            note: Some(69),
            cents: Some(0),
            buffer: MonoSample::default(),
            crossfade_buffer: MonoSample::default(),
        }
    }
}

fn load_sample(path: &Path) -> Result<Sample, LoadingError> {
    let mut file = File::open(path).context(IOSnafu)?;
    let mut data = String::new();
    file.read_to_string(&mut data).context(IOSnafu)?;

    serde_json::from_str(&data).context(ParserSnafu)
}

fn save_sample_metadata(path: &Path, metadata: &Sample) -> Result<(), LoadingError> {
    fs::write(path, serde_json::to_string_pretty(metadata).context(ParserSnafu)?).context(IOSnafu)
}

impl Resource for Sample {
    fn load_resource(path: &Path) -> Result<Self, LoadingError>
    where
        Self: Sized,
    {
        // next, get the sample metadata (if it exists)
        let mut sample: Sample = Sample::default();
        let metadata_path = path.with_extension("json");
        if let Ok(does_exist) = path.with_extension("json").try_exists() {
            if does_exist {
                sample = load_sample(&metadata_path)?;
            } else {
                save_sample_metadata(&metadata_path, &sample)?;
            }
        }

        let file = BufReader::new(File::open(path).context(IOSnafu)?);
        let source = Decoder::new(file).unwrap();

        let sample_rate = source.sample_rate();
        let channels = source.channels();
        let length = source.size_hint().0;

        let mut buffer: Vec<f32> = Vec::with_capacity(length);
        buffer.extend(source.map(|x| x as f32 / i16::MAX as f32).step_by(channels as usize));

        sample.buffer = MonoSample {
            audio_raw: buffer,
            sample_rate,
        };

        if let (Some(loop_start), Some(loop_end)) = (sample.loop_start, sample.loop_end) {
            if sample.crossfade > 0 {
                // calculate crossfade here
                let mut crossfade_audio: Vec<f32> = Vec::new();

                let audio = &sample.buffer.audio_raw;

                for i in 0..sample.crossfade {
                    crossfade_audio.push(lerp(
                        audio[loop_end + i],
                        audio[loop_start + i],
                        i as f32 / sample.crossfade as f32,
                    ));
                }

                sample.crossfade_buffer = MonoSample {
                    audio_raw: crossfade_audio,
                    sample_rate,
                }
            }
        }

        Ok(sample)
    }
}
