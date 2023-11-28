use std::{
    collections::BTreeMap,
    fs::{self, remove_file},
    sync::{Arc, Mutex},
};

use common::resource_manager::ResourceId;
use futures::future::join_all;
use rfd::AsyncFileDialog;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use sound_engine::{
    sampling::{
        envelope::{calc_sample_metadata, SampleMetadata},
        phase_calculator::PhaseCalculator,
        pipe_player::{envelope_indexes, EnvelopeType},
        rank::{Pipe, Rank},
    },
    MonoSample,
};

use crate::{
    errors::{EngineError, IoSnafu, JsonParserSnafu},
    resource::{
        rank::{expected_sample_location, RankConfig},
        sample::{check_for_note_number, load_sample},
    },
    routes::{prelude::*, RouteReturn},
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    file_name: String,
    rank_name: String,
}

pub async fn route<'a>(mut state: RouteState<'a>) -> Result<RouteReturn, EngineError> {
    let Payload { file_name, rank_name } =
        serde_json::from_value(state.msg["payload"].take()).context(JsonParserSnafu)?;

    let files = AsyncFileDialog::new().set_file_name("untitled.mjuo").pick_files().await;

    let sample_directory = state
        .global_state
        .project_directory()
        .unwrap()
        .join("samples")
        .join(&file_name);

    let rank_file = state
        .global_state
        .project_directory()
        .unwrap()
        .join("ranks")
        .join(format!("{}.toml", file_name));

    fs::create_dir_all(&sample_directory).context(IoSnafu)?;

    if let Some(files) = files {
        let calculated: Arc<Mutex<Vec<(SampleMetadata, String, MonoSample, u8)>>> = Arc::new(Mutex::new(vec![]));

        join_all(files.into_iter().map(|file| {
            let calculated_clone = calculated.clone();
            let sample_directory = sample_directory.clone();

            tokio::spawn(async move {
                let path = file.path();
                let sample = load_sample(path);

                if let Ok(sample) = sample {
                    let note_number = path
                        .file_stem()
                        .and_then(|stem| check_for_note_number(&stem.to_string_lossy()));
                    let possible_freq = note_number.map(|note| 440.0 * 2_f64.powf((note as i16 - 69) as f64 / 12.0));

                    let metadata = calc_sample_metadata(&sample.audio_raw, sample.sample_rate, possible_freq);
                    let note = note_number.unwrap_or(metadata.closest_note);

                    // write the file as wav
                    let spec = hound::WavSpec {
                        channels: 1,
                        sample_rate: sample.sample_rate,
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    };

                    let filename = expected_sample_location(note, "wav");
                    let file_location = sample_directory.join(&filename);

                    if file_location.exists() {
                        remove_file(&file_location).unwrap();
                    }

                    let mut writer = hound::WavWriter::create(file_location, spec).unwrap();

                    for frame in &sample.audio_raw {
                        writer.write_sample(*frame).unwrap();
                    }

                    writer.finalize().unwrap();

                    calculated_clone
                        .lock()
                        .unwrap()
                        .push((metadata, filename, sample, note));
                }
            })
        }))
        .await;

        let mut samples = Arc::try_unwrap(calculated).unwrap().into_inner().unwrap();
        let mut pipes: BTreeMap<u8, Pipe> = BTreeMap::new();

        samples.sort_by_key(|sample| sample.3);

        for (metadata, filename, sample, note) in samples.into_iter() {
            let buffer_rate = sample.sample_rate;
            let amp_window_size = (buffer_rate as f32 / metadata.freq as f32) as usize * 2;

            let phase_calculator = PhaseCalculator::new(metadata.freq as f32, buffer_rate);

            let attack_envelope = envelope_indexes(
                metadata.decay_index,
                metadata.release_index,
                &sample,
                amp_window_size,
                EnvelopeType::Attack,
            );
            let release_envelope = envelope_indexes(
                metadata.decay_index,
                metadata.release_index,
                &sample,
                amp_window_size,
                EnvelopeType::Release,
            );

            let pipe = Pipe {
                freq: metadata.freq as f32,
                resource: ResourceId {
                    namespace: "samples".into(),
                    resource: filename,
                },

                amplitude: 1.0,
                comb_coeff: 0.0,

                crossfade: 256,
                loop_start: metadata.loop_start,
                loop_end: metadata.loop_end,
                decay_index: metadata.decay_index,
                release_index: metadata.release_index,

                amp_window_size,
                phase_calculator,
                attack_envelope: attack_envelope,
                release_envelope: release_envelope,
            };

            pipes.insert(note, pipe);
        }

        let rank = Rank { pipes, name: rank_name };
        let config = RankConfig::from_rank(
            rank,
            ResourceId {
                namespace: "samples".into(),
                resource: file_name,
            },
        );

        println!("\n\nnew config: {:?}\n\n", config);

        let rank = toml_edit::ser::to_string_pretty(&config).unwrap();
        fs::write(rank_file, rank).unwrap();
    }

    Ok(RouteReturn::default())
}
