use std::error::Error;
use std::io::Read;

use alsa::{Direction, Rawmidi};

use crate::io::MidiClientBackend;

pub struct AlsaMidiClientBackend {
    client: Option<Rawmidi>,
}

impl AlsaMidiClientBackend {
    pub fn new() -> AlsaMidiClientBackend {
        AlsaMidiClientBackend { client: None }
    }
}

impl MidiClientBackend for AlsaMidiClientBackend {
    fn read(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut out = [0_u8; 512];

        let bytes_result = if let Some(client) = &self.client {
            client.io().read(&mut out)
        } else {
            return Err("Midi backend not initialized".into());
        };

        let bytes_read = match bytes_result {
            Ok(bytes) => bytes,
            Err(error) => {
                if let Some(err) = error.raw_os_error() {
                    if err == -11 {
                        0_usize // there was nothing to read
                    } else {
                        return Err(Box::new(error));
                    }
                } else {
                    return Err(Box::new(error));
                }
            }
        };

        let mut buffer = vec![0; bytes_read];

        buffer[..bytes_read].clone_from_slice(&out[..bytes_read]);

        Ok(buffer)
    }

    fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        self.client = Some(Rawmidi::new("virtual", Direction::Capture, true)?);

        Ok(())
    }
}

impl Default for AlsaMidiClientBackend {
    fn default() -> AlsaMidiClientBackend {
        AlsaMidiClientBackend::new()
    }
}
