use std::fs::File;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug)]
struct WavFmtHeader {
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

pub struct MonoWav {
    audio_raw: Vec<f32>,
    sample_length: usize,
    sample_rate: u32,
}

fn read_wav_as_mono<P: AsRef<Path>>(path: P) -> Result<MonoWav, Error>  {
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

    let sample_count = ((file_length - data_start) / (fmt_header.block_align as u64)) as usize;

    let mut sample = vec![0_f32; sample_count];
    let mut buffer = vec![0_u8; fmt_header.block_align as usize];

    while file.seek(SeekFrom::Current(0))? > 0 {
        // mix down to mono
        file.read_exact(buffer.as_mut_slice())?;

        match fmt_header.bits_per_sample {
            8 => {
                let mut sample_result = 0_f32;

                for i in 0..(fmt_header.channels as usize) {
                    sample_result += (((buffer[i] - 128) as f32 / u8::MAX as f32) * 2.0) - 1.0;
                }

                sample.push(sample_result / fmt_header.channels as f32);
            },
            16 => {
                let mut sample_result = 0_f32;

                for (i, channel) in buffer.chunks_exact(2).enumerate() {
                    sample_result += i16::from_le_bytes(channel.try_into().unwrap_or_default()) as f32 / i16::MAX as f32;
                }

                sample.push(sample_result / fmt_header.channels as f32);
            }, // i'm too lazy to implement the rest
            bps => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("I- I don't even know how I got here. You're using a bits per sample of {}???", bps)
                ));
            }
        }
    }

    Ok(MonoWav {
        audio_raw: sample,
        sample_length: sample.len(),
        sample_rate: fmt_header.sample_rate
    })
}