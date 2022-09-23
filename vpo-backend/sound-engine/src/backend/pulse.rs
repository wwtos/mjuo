use std::error::Error;

use psimple::Simple;
use pulse::sample::Spec;
use pulse::stream::Direction;

use crate::backend::AudioClientBackend;
use crate::constants::{BUFFER_SIZE, SAMPLE_RATE};

pub struct PulseClientBackend {
    pub pulse_spec: Option<pulse::sample::Spec>,
    pub client: Option<psimple::Simple>,
}

impl PulseClientBackend {
    pub fn new() -> PulseClientBackend {
        PulseClientBackend {
            pulse_spec: None,
            client: None,
        }
    }
}

impl AudioClientBackend for PulseClientBackend {
    fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let spec = Spec {
            format: pulse::sample::Format::F32le,
            channels: 1,
            rate: SAMPLE_RATE,
        };
        assert!(spec.is_valid());

        let s = Simple::new(
            None,                // Use the default server
            "Synthesizer Test",  // Our application’s name
            Direction::Playback, // We want a playback stream
            None,                // Use the default device
            "Music",             // Description of our stream
            &spec,               // Our sample format
            None,                // Use default channel map
            None,                // Use default buffering attributes
        )?;

        self.pulse_spec = Some(spec);
        self.client = Some(s);

        Ok(())
    }

    fn write(&mut self, data: &[f32; BUFFER_SIZE]) -> Result<(), Box<dyn Error>> {
        let mut data_out = [0_u8; BUFFER_SIZE * 4];

        // TODO: would memcpy work here faster?
        for i in 0..BUFFER_SIZE {
            // if data[i] > 1.0 || data[i] < -1.0 {
            //     print!("Clipping!");
            // }

            let num = data[i].to_le_bytes();

            data_out[i * 4] = num[0];
            data_out[i * 4 + 1] = num[1];
            data_out[i * 4 + 2] = num[2];
            data_out[i * 4 + 3] = num[3];
        }

        match &self.client {
            Some(client) => client.write(&data_out),
            None => unimplemented!(),
        }?;

        Ok(())
    }

    fn drain(&self) -> Result<(), Box<dyn Error>> {
        match &self.client {
            Some(client) => client.drain(),
            None => unimplemented!(),
        }?;

        Ok(())
    }
}

impl Default for PulseClientBackend {
    fn default() -> PulseClientBackend {
        PulseClientBackend::new()
    }
}
