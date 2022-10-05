use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use rodio::{Decoder, Source};
use serde::{Deserialize, Serialize};

use crate::SamplePoint;

#[derive(Serialize, Deserialize)]
pub struct Sample {
    pub loop_start: Option<usize>,
    pub loop_end: Option<usize>,
    pub release: Option<usize>,
    #[serde(skip)]
    pub buffer: Vec<SamplePoint>,
    #[serde(skip)]
    pub sample_rate: u32,
}

pub struct AudioLoader {
    loaded_samples: Vec<Option<Sample>>,
    registry: HashMap<PathBuf, usize>,
}

impl AudioLoader {
    pub fn new() -> AudioLoader {
        AudioLoader {
            loaded_samples: Vec::new(),
            registry: HashMap::new(),
        }
    }

    pub fn load(&mut self, path: &Path) -> Result<usize, io::Error> {
        if let Some(index) = self.registry.get(&PathBuf::from(fs::canonicalize(path)?)) {
            Ok(*index)
        } else {
            let file = BufReader::new(File::open(path)?);
            let source = Decoder::new(file).unwrap();

            let sample_rate = source.sample_rate();
            let buffer: Vec<SamplePoint> = source.collect();

            let sample_info_path = path.with_extension("json");
            let sample_info_raw = fs::read_to_string(sample_info_path)?;
            let mut sample: Sample = serde_json::from_str(&sample_info_raw)?;

            sample.buffer = buffer;
            sample.sample_rate = sample_rate;

            self.loaded_samples.push(Some(sample));
            self.registry.insert(path.into(), self.loaded_samples.len() - 1);

            Ok(self.loaded_samples.len() - 1)
        }
    }

    pub fn get_sample(&self, index: usize) -> Option<&Sample> {
        self.loaded_samples[index].as_ref()
    }
}
