use crate::errors::SymphoniaSnafu;
use snafu::ResultExt;
use symphonia::core::{
    audio::{SampleBuffer, SignalSpec},
    codecs::DecoderOptions,
    errors::Error,
    formats::FormatOptions,
    io::{MediaSource, MediaSourceStream},
    meta::MetadataOptions,
    probe::Hint,
};

use crate::errors::EngineError;

pub fn decode_audio(source: Box<dyn MediaSource>, hint: Hint) -> Result<(Vec<f32>, SignalSpec), EngineError> {
    let mss = MediaSourceStream::new(source, Default::default());

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .context(SymphoniaSnafu)?;

    let mut format = probed.format;
    let track_def = format.default_track().unwrap();

    let mut decoder = symphonia::default::get_codecs()
        .make(&track_def.codec_params, &decoder_opts)
        .context(SymphoniaSnafu)?;

    let track_id = track_def.id;

    let mut sample_buf = None;
    let mut sample_spec = None;

    let mut result_buf: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };

        // If the packet does not belong to the selected track, skip it.
        if packet.track_id() != track_id {
            continue;
        }

        // Decode the packet into audio samples, ignoring any decode errors.
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                if sample_buf.is_none() {
                    // Get the audio buffer specification.
                    let spec = *audio_buf.spec();

                    // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                    let duration = audio_buf.capacity() as u64;

                    // Create the f32 sample buffer.
                    sample_spec = Some(spec);
                    sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                }

                // Copy the decoded audio buffer into the sample buffer in an interleaved format.
                if let Some(buf) = &mut sample_buf {
                    buf.copy_interleaved_ref(audio_buf);
                    result_buf.extend(buf.samples().iter());
                }
            }
            Err(Error::DecodeError(_)) => (),
            Err(error) => return Err(EngineError::SymphoniaError { source: error }),
        }
    }

    match sample_spec {
        Some(spec) => Ok((result_buf, spec)),
        None => Err(EngineError::AudioParserError),
    }
}

pub fn mix_to_mono(audio: &Vec<f32>, channel_count: usize) -> Vec<f32> {
    let duration = audio.len() / channel_count;
    let mut result: Vec<f32> = Vec::with_capacity(duration);

    // mix to mono
    for i in (0..duration).step_by(channel_count) {
        let mut sum = 0.0;

        for j in 0..channel_count {
            sum += audio[i + j];
        }

        result.push(sum);
    }

    result
}
