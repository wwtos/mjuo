use sound_engine::wave::wavetable::Wavetable;
use symphonia::core::{io::MediaSource, probe::Hint};

use crate::errors::EngineError;

use super::util::{decode_audio, mix_to_mono};

pub fn load_wavetable(resource: Box<dyn MediaSource>) -> Result<Wavetable, EngineError> {
    // next, get the wavetable metadata (if it exists)
    let mut wavetable: Wavetable = Wavetable::default();

    let (buffer, spec) = decode_audio(resource, Hint::new())?;
    let channels = spec.channels.count();

    wavetable.wavetable = mix_to_mono(&buffer, channels);

    Ok(wavetable)
}
