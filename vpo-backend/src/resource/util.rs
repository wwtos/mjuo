pub fn mix_to_mono(audio: &Vec<f32>, channel_count: usize) -> Vec<f32> {
    let duration = audio.len() / channel_count;
    let mut result: Vec<f32> = Vec::with_capacity(duration);

    // mix to mono
    for i in (0..audio.len()).step_by(channel_count) {
        let mut sum = 0.0;

        for j in 0..channel_count {
            sum += audio[i + j];
        }

        result.push(sum);
    }

    result
}
