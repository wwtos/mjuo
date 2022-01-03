use crate::node::{AudioNode, InputType, OutputType};
use crate::{error::NodeError, error::NodeErrorType};

use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use crate::ringbuffer::RingBuffer;

pub struct WavReader {
    output_out: f32,
    file_opened: Option<File>,
    wav_header: Option<WavFmtHeader>,
    audio_raw: Option<Rc<RefCell<Vec<f32>>>>
}

#[derive(Debug)]
pub struct WavFmtHeader {
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

impl WavReader {
    pub fn new() -> WavReader {
        WavReader {
            output_out: 0_f32,
            file_opened: None,
            wav_header: None,
            audio_raw: None
        }
    }
}

// (elephant paper) http://yehar.com/blog/wp-content/uploads/2009/08/deip.pdf
// https://stackoverflow.com/questions/1125666/how-do-you-do-bicubic-or-other-non-linear-interpolation-of-re-sampled-audio-da
fn hermite_interpolate(x0: f32, x1: f32, x2: f32, x3: f32, t: f32) -> f32 {
    let c0 = x1;
    let c1 = 0.5 * (x2 - x0);
    let c2 = x0 - (2.5 * x1) + (2.0 * x2) - (0.5 * x3);
    let c3 = (0.5 * (x3 - x0)) + (1.5 * (x1 - x2));
    return (((((c3 * t) + c2) * t) + c1) * t) + c0;
}

impl WavReader {
    pub fn open<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let mut file = File::open(path)?;

        let mut four_byte_buffer = [0_u8; 4];
        let mut two_byte_buffer = [0_u8; 2];

        // read headers first (https://sites.google.com/site/musicgapi/technical-documents/wav-file-format)

        // first four bytes should be "RIFF"
        file.read_exact(&mut four_byte_buffer)?;

        if b"RIFF" != &four_byte_buffer {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "WAV has malformed data.",
            ));
        }

        // next is the file size
        file.read_exact(&mut four_byte_buffer)?;
        let file_length = u32::from_le_bytes(four_byte_buffer) as u64;

        // next is RIFF type (should be WAVE)
        file.read_exact(&mut four_byte_buffer)?;

        if b"WAVE" != &four_byte_buffer {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "WAV has malformed data.",
            ));
        }

        // now we're done with the main part, next is parsing out the headers
        let mut fmt_header = WavFmtHeader {
            channels: 0,
            sample_rate: 0,
            byte_rate: 0,
            block_align: 0,
            bits_per_sample: 0,
        };

        let mut data_start = 0;

        loop {
            // read subchunk ID (type)
            file.read_exact(&mut four_byte_buffer)?;

            let id = four_byte_buffer;

            match &id {
                b"fmt " => {
                    file.read_exact(&mut four_byte_buffer)?; // get length of chunk

                    if u32::from_le_bytes(four_byte_buffer) != 16 {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "WAV has malformed data.",
                        ));
                    }

                    // we only support PCM uncompressed format
                    file.read_exact(&mut two_byte_buffer)?;

                    if u16::from_le_bytes(two_byte_buffer) != 1 {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Cannot read WAV other than type 1 (uncompressed PCM)",
                        ));
                    }

                    // # of channels
                    file.read_exact(&mut two_byte_buffer)?;
                    fmt_header.channels = u16::from_le_bytes(two_byte_buffer);

                    // sample rate
                    file.read_exact(&mut four_byte_buffer)?;
                    fmt_header.sample_rate = u32::from_le_bytes(four_byte_buffer);

                    // bytes per second
                    file.read_exact(&mut four_byte_buffer)?;
                    fmt_header.byte_rate = u32::from_le_bytes(four_byte_buffer);

                    // block align
                    file.read_exact(&mut two_byte_buffer)?;
                    fmt_header.block_align = u16::from_le_bytes(two_byte_buffer);

                    // bits per sample
                    file.read_exact(&mut two_byte_buffer)?;
                    fmt_header.bits_per_sample = u16::from_le_bytes(two_byte_buffer);
                }
                b"data" => {
                    // we reached the data, stop looping over info
                    file.seek(SeekFrom::Current(4))?;
                    data_start = file.seek(SeekFrom::Current(0))?;

                    break;
                }
                // we don't care, so jump to the end
                _ => {
                    file.read_exact(&mut four_byte_buffer)?; // get length of chunk
                                                             // jump ahead
                    file.seek(SeekFrom::Current(
                        u32::from_le_bytes(four_byte_buffer) as i64
                    ))?;
                }
            }
        }

        self.past_samples = RingBuffer::new(fmt_header.byte_rate as usize * 4, 0.0);
        self.wav_header = Some(fmt_header);
        self.data_start = data_start;
        self.file_opened = Some(file);
        self.file_length = file_length;
        self.current_sample_position = 0;

        Ok(())
    }

    pub fn available(&mut self) -> Result<u64, Error> {
        if let Some(file) = &mut self.file_opened {
            Ok(self.file_length - file.seek(SeekFrom::Current(0))?)
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Wav reader hasn't been opened yet!",
            ))
        }
    }

    pub fn read_one_sample(&mut self) -> Result<Vec<f32>, Error> {
        if let Some(file) = &mut self.file_opened {
            let wav_header = self.wav_header.as_ref().unwrap();

            let mut buffer = vec![0_u8; wav_header.block_align as usize];
            file.read_exact(buffer.as_mut_slice())?;

            self.current_sample_position += 1;

            match wav_header.bits_per_sample {
                8 => {
                    let mut channel_result = vec![0_f32; wav_header.channels as usize];

                    for i in 0..(wav_header.channels as usize) {
                        channel_result[i] = (buffer[i] - 128) as f32 / u8::MAX as f32;
                    }

                    Ok(channel_result)
                },
                16 => {
                    let mut channel_result = vec![0_f32; wav_header.channels as usize];

                    for (i, channel) in buffer.chunks_exact(2).enumerate() {
                        channel_result[i] = i16::from_le_bytes(channel.try_into().unwrap_or_default()) as f32 / i16::MAX as f32;
                    }

                    Ok(channel_result)
                }, // i'm too lazy to implement the rest
                bps => {
                    Err(Error::new(
                        ErrorKind::Other,
                        format!("I- I don't even know how I got here. You're using a bits per sample of {}???", bps)
                    ))
                }
            }
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Wav reader hasn't been opened yet!",
            ))
        }
    }

    pub fn seek_by_sample(&mut self, sample_position: u64) -> Result<(), Error> {
        if let Some(file) = &mut self.file_opened {
            let wav_header = self.wav_header.as_ref().unwrap();

            let byte_position = sample_position / (wav_header.byte_rate as u64) + self.data_start;

            if byte_position > self.file_length || byte_position < self.data_start {
                Err(Error::new(
                    ErrorKind::Other,
                    "Seeking out of bounds of file!",
                ))
            } else {
                file.seek(SeekFrom::Start(byte_position))?;
                self.current_sample_position += 1;
                self.current_time_position = self.current_sample_position as f32 * self.playback_speed;

                Ok(())
            }
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Wav reader hasn't been opened yet!",
            ))
        }
    }

    pub fn next(&mut self) -> Result<Vec<f32>, Error> {
        if let Some(wav_header) = &mut self.wav_header {
            if !self.past_samples_filled {
                // load it with first four samples
                for _ in 0..4 {
                    let sample = self.read_one_sample()?;

                    sample
                        .into_iter()
                        .for_each(|x| self.past_samples.push_end(x));
                }

                self.past_samples_filled = true;
                // this first time through the buffer hasn't been established, so
                // we need to wait until next sample requested before outputting audio

                return Ok(vec![0_f32; wav_header.channels as usize]);
            } else {
                // load next sample
                let sample = self.read_one_sample()?;

                sample
                    .into_iter()
                    .for_each(|x| self.past_samples.push_end(x));
            }

            for channel in 0..wav_header.channels {
                hermite_interpolate(
                    self.past_samples.get(channel as usize),
                    self.past_samples.get((channel + wav_header.channels) as usize),
                    self.past_samples.get((channel + wav_header.channels * 2) as usize),
                    self.past_samples.get((channel + wav_header.channels * 3) as usize),
                );
            }

            Ok(Vec::new())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Wav reader hasn't been opened yet!",
            ))
        }
    }

    pub fn get_output_audio(&self) -> f32 {
        self.output_out
    }
}

impl AudioNode for WavReader {
    fn process(&mut self) {}

    fn receive_audio(&mut self, input_type: InputType, _: f32) -> Result<(), NodeError> {
        Err(NodeError::new(
            format!("Envelope cannot input audio of type {:?}", input_type),
            NodeErrorType::UnsupportedInput,
        ))
    }

    fn get_output_audio(&self, output_type: OutputType) -> Result<f32, NodeError> {
        match output_type {
            OutputType::Out => Ok(self.output_out),
            _ => Err(NodeError::new(
                format!("Envelope cannot output audio of type {:?}", output_type),
                NodeErrorType::UnsupportedOutput,
            )),
        }
    }

    fn list_inputs(&self) -> Vec<InputType> {
        vec![InputType::In]
    }

    fn list_outputs(&self) -> Vec<OutputType> {
        vec![OutputType::Out]
    }
}

impl Default for WavReader {
    fn default() -> Self {
        Self::new()
    }
}
