use std::io::Error;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::util::wav_reader::read_wav_as_mono;
use crate::MonoSample;

pub struct SampleManager {
    samples: HashMap<String, Rc<RefCell<MonoSample>>>,
}

impl SampleManager {
    pub fn load_sample(&mut self, sample_path: &str) -> Result<(), Error> {
        if self.samples.contains_key(sample_path) {
            return Ok(()); // it was already loaded, no need to worry
        }

        let sample = read_wav_as_mono(sample_path)?;

        self.samples
            .insert(sample_path.to_string(), Rc::new(RefCell::new(sample)));

        Ok(())
    }

    pub fn get_sample_lazy(&mut self, sample_path: &str) -> Rc<RefCell<MonoSample>> {
        match self.load_sample(sample_path) {
            Ok(_) => self.samples.get(sample_path).unwrap().clone(),
            Err(_) => Rc::new(RefCell::new(MonoSample {
                sample_rate: 44_100,
                audio_raw: vec![0.0; 1],
            })),
        }
    }
}
