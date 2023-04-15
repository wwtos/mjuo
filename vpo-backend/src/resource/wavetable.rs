use byteorder::{LittleEndian, ReadBytesExt};
use sound_engine::wave::wavetable::Wavetable;

use crate::errors::EngineError;

pub fn load_wavetable(sample: Vec<u8>) -> Result<Wavetable, EngineError> {
    // next, get the wavetable metadata (if it exists)
    let mut wavetable: Wavetable = Wavetable::default();

    let mut buffer: Vec<f32> = Vec::with_capacity(sample.len() / 4);

    for i in (0..sample.len()).step_by(4) {
        let mut frame = &sample[i..(i + 4)];
        buffer.push(frame.read_f32::<LittleEndian>().unwrap());
    }

    wavetable.wavetable = buffer;

    Ok(wavetable)
}
