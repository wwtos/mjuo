use crate::node::{AudioNode, InputType, OutputType};
use crate::{error::NodeError, error::NodeErrorType};

use std::fs::File;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

pub struct WavReader {
    output_out: f32,
    file_opened: Option<File>,
    wav_header: Option<WavFmtHeader>,
    data_start: u64,
    file_length: u64,
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
            data_start: 0,
            file_length: 0,
        }
    }
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

        self.wav_header = Some(fmt_header);
        self.data_start = data_start;
        self.file_opened = Some(file);
        self.file_length = file_length;

        Ok(())
    }

    pub fn read_one_sample(&mut self) -> Result<Vec<f32>, Error> {
        if let Some(file) = &mut self.file_opened {
            let wav_header = self.wav_header.as_ref().unwrap();

            let mut buffer = vec![0_u8; wav_header.block_align as usize];
            file.read_exact(buffer.as_mut_slice())?;

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
