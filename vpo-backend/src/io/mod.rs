pub mod cpal;
pub mod midir;

use std::collections::HashMap;

use std::fmt::Debug;
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;

use lazy_static::lazy_static;

use node_engine::{global_state::GlobalState, state::NodeState};
use resource_manager::ResourceManager;
use semver::Version;
use serde_json::{json, Value};
use snafu::ResultExt;
use threadpool::ThreadPool;
use walkdir::WalkDir;

use crate::errors::{IoSnafu, JsonParserSnafu};

use crate::errors::EngineError;
use crate::migrations::migrate;
use crate::resource::rank::load_rank_from_file;
use crate::resource::sample::load_sample;

pub mod midi;

const AUDIO_EXTENSIONS: &'static [&'static str] = &["ogg", "wav", "mp3", "flac"];
lazy_static! {
    pub static ref VERSION: Version = Version::parse("0.4.0").unwrap();
}

pub fn save(state: &NodeState, path: &Path) -> Result<(), EngineError> {
    let state = json!({
        "version": VERSION.to_string(),
        "state": state.to_json()
    });

    fs::write(
        path.join("state.json"),
        serde_json::to_string_pretty(&state).context(JsonParserSnafu)?,
    )
    .context(IoSnafu)?;

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

pub fn load(path: &Path, state: &mut NodeState, global_state: &mut GlobalState) -> Result<(), EngineError> {
    let json_raw = fs::read_to_string(path.join("state.json")).context(IoSnafu)?;
    let json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    if let Some(version) = json["version"].as_str() {
        if version != VERSION.to_string() {
            migrate(PathBuf::from(path))?;
        }
    }

    let json_raw = fs::read_to_string(path.join("state.json")).context(IoSnafu)?;
    let mut json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    *state = NodeState::new(global_state).unwrap();
    global_state.reset();

    let mut resources = global_state.resources.write().unwrap();

    load_resources(
        &path.join("samples"),
        &mut resources.samples,
        AUDIO_EXTENSIONS,
        &load_sample,
    )?;
    load_resources(
        &path.join("ranks"),
        &mut resources.ranks,
        &["toml"],
        &load_rank_from_file,
    )?;

    let graph_manager = serde_json::from_value(json["graph_manager"].take()).context(JsonParserSnafu)?;
    let root_graph_index = serde_json::from_value(json["root_graph_index"].take()).context(JsonParserSnafu)?;
    let output_node = serde_json::from_value(json["output_node"].take()).context(JsonParserSnafu)?;
    let midi_in_node = serde_json::from_value(json["midi_in_node"].take()).context(JsonParserSnafu)?;

    state.load_state(graph_manager, root_graph_index, output_node, midi_in_node);

    Ok(())
}
