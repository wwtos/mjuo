pub mod midir;
#[cfg(target_os = "linux")]
pub mod pulse;

pub mod cpal;

use std::collections::HashMap;
use std::error::Error;

use std::fmt::Debug;
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;

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
use threadpool::ThreadPool;
use walkdir::WalkDir;

use crate::errors::EngineError;
use crate::migrations::migrate;
use crate::resource::rank::load_rank_from_file;
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
    load_resource: &'static F,
) -> Result<(), EngineError>
where
    T: Send + Sync + Debug + 'static,
    F: Fn(PathBuf) -> Result<T, EngineError> + Send + Sync,
{
    // build iterator to traverse directories
    let asset_list = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| {
            if let Ok(res) = e {
                if res.metadata().unwrap().is_file() {
                    // only resources with extensions in `valid_extensions` are allowed
                    if let Some(extension) = res.path().extension() {
                        if valid_extensions.contains(&extension.to_string_lossy().as_ref()) {
                            return Some(res);
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

    // spawn threads to load everything
    let new_resources: Arc<Mutex<HashMap<String, T>>> = Arc::new(Mutex::new(HashMap::new()));
    let pool = ThreadPool::new(available_parallelism().unwrap_or(NonZeroUsize::new(4).unwrap()).into());

    for asset in asset_list {
        let resources_ref = Arc::clone(&new_resources);
        let (key, location) = asset.clone();

        pool.execute(move || {
            // load and register it
            let new_resource = load_resource(location.clone()).unwrap();
            println!("Loaded: {}", location.to_string_lossy());

            resources_ref.lock().unwrap().insert(key, new_resource);
        });
    }

    pool.join();

    let new_resources = Arc::try_unwrap(new_resources).unwrap().into_inner().unwrap();

    for (key, resource) in new_resources.into_iter() {
        resources.add_resource(key, resource);
    }

    Ok(())
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
    global_state.resources.pipes.clear();

    load_resources(
        &path.join("samples"),
        &mut global_state.resources.pipes,
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
        &load_rank_from_file,
    )
    .context(LoadingSnafu)?;

    state.apply_json(json["state"].take())?;

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
