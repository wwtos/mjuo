#[cfg(target_os = "linux")]
pub mod alsa;
#[cfg(target_os = "linux")]
pub mod alsa_midi;
pub mod midir;
#[cfg(target_os = "linux")]
pub mod pulse;

pub mod cpal;

use std::error::Error;

use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;

use node_engine::errors::LoadingSnafu;
use node_engine::{
    errors::{IOSnafu, JsonParserSnafu, NodeError},
    global_state::GlobalState,
    state::NodeEngineState,
};
use resource_manager::{LoadingError, ResourceManager};
use semver::Version;
use serde_json::{json, Value};
use snafu::ResultExt;
use walkdir::WalkDir;

use crate::migrations::migrate;
use crate::resource::rank::load_rank;
use crate::resource::sample::load_sample;
use crate::resource::wavetable::load_wavetable;

pub mod midi;

pub const BUFFER_SIZE: usize = 256;
pub const SAMPLE_RATE: u32 = 48_000;

const AUDIO_EXTENSIONS: &'static [&'static str] = &["ogg", "wav", "mp3", "flac"];
lazy_static! {
    pub static ref VERSION: Version = Version::parse("0.4.0").unwrap();
}

pub fn save(state: &NodeEngineState, path: &Path) -> Result<(), NodeError> {
    let state = json!({
        "version": VERSION.to_string(),
        "state": state.to_json()?
    });

    fs::write(
        path.join("state.json"),
        serde_json::to_string_pretty(&state).context(JsonParserSnafu)?,
    )
    .context(IOSnafu)?;

    Ok(())
}

fn load_resources<T, F>(
    path: &Path,
    resources: &mut ResourceManager<T>,
    valid_extensions: &[&str],
    load_sample: &'static F,
) -> Result<(), LoadingError>
where
    T: Send + Sync + Debug + 'static,
    F: Fn(PathBuf) -> Result<T, LoadingError> + Send + Sync,
{
    let asset_list = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| {
            if let Ok(res) = e {
                if res.metadata().unwrap().is_file() {
                    if let Some(extension) = res.path().extension() {
                        if let Some(extension) = extension.to_str() {
                            if valid_extensions.contains(&extension) {
                                return Some(res);
                            }
                        }
                    }
                }
            }

            None
        })
        .map(|asset| {
            let asset_key = asset.path().strip_prefix(path).unwrap().to_string_lossy().to_string();
            (asset_key, PathBuf::from(asset.path()))
        });

    resources.watch_resources(asset_list, load_sample)
}

pub fn load(path: &Path, state: &mut NodeEngineState, global_state: &mut GlobalState) -> Result<(), NodeError> {
    let json_raw = fs::read_to_string(path.join("state.json")).context(IOSnafu)?;
    let json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    if let Some(version) = json["version"].as_str() {
        if version != VERSION.to_string() {
            migrate(PathBuf::from(path))?;
        }
    }

    let json_raw = fs::read_to_string(path.join("state.json")).context(IOSnafu)?;
    let mut json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    *state = NodeEngineState::new(global_state).unwrap();
    global_state.resources.samples.clear();

    load_resources(
        &path.join("samples"),
        &mut global_state.resources.samples,
        AUDIO_EXTENSIONS,
        &load_sample,
    )
    .context(LoadingSnafu)?;
    load_resources(
        &path.join("wavetables"),
        &mut global_state.resources.wavetables,
        AUDIO_EXTENSIONS,
        &load_wavetable,
    )
    .context(LoadingSnafu)?;
    load_resources(
        &path.join("ranks"),
        &mut global_state.resources.ranks,
        &["toml"],
        &load_rank,
    )
    .context(LoadingSnafu)?;

    // TODO: version handling and migrations here
    state.apply_json(json["state"].take(), global_state)?;

    Ok(())
}

pub trait AudioClientBackend {
    fn write(&mut self, data: &[f32; BUFFER_SIZE]) -> Result<(), Box<dyn Error>>;
    fn connect(&mut self) -> Result<(), Box<dyn Error>>;
    fn drain(&self) -> Result<(), Box<dyn Error>>;
}

pub trait MidiClientBackend {
    fn read(&self) -> Result<Vec<u8>, Box<dyn Error>>;
    fn connect(&mut self) -> Result<(), Box<dyn Error>>;
}
