pub mod cpal;
pub mod file_watcher;
pub mod midir;

use std::collections::HashMap;

use std::fmt::Debug;
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::available_parallelism;

use lazy_static::lazy_static;

use node_engine::{global_state::GlobalState, state::NodeState};
use notify::{Config, Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use resource_manager::{ResourceIndex, ResourceManager};
use semver::Version;
use serde_json::{json, Value};
use snafu::{OptionExt, ResultExt};
use threadpool::ThreadPool;
use walkdir::WalkDir;

use crate::errors::{IoSnafu, JsonParserSnafu};

use crate::errors::EngineError;
use crate::migrations::migrate;
use crate::resource::rank::load_rank_from_file;
use crate::resource::sample::load_sample;

const AUDIO_EXTENSIONS: &[&str] = &["ogg", "wav", "mp3", "flac"];
lazy_static! {
    pub static ref VERSION: Version = Version::parse("0.4.0").unwrap();
}

pub fn save(state: &NodeState, path: &Path) -> Result<(), EngineError> {
    let state = json!({
        "version": VERSION.to_string(),
        "state": state.to_json()
    });

    let parent = path
        .parent()
        .whatever_context(format!("path {:?} has no parent", path))?;

    fs::create_dir_all(parent.join("samples")).context(IoSnafu)?;
    fs::create_dir_all(parent.join("ranks")).context(IoSnafu)?;
    fs::write(path, serde_json::to_string_pretty(&state).context(JsonParserSnafu)?).context(IoSnafu)?;

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
    F: Fn(&Path) -> Result<T, EngineError> + Send + Sync,
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
            let new_resource = load_resource(&location).unwrap();
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

pub fn load_single(file: &Path, global_state: &mut GlobalState) -> Result<(), EngineError> {
    let root = global_state.active_project.as_ref().and_then(|x| x.parent()).unwrap();

    let relative_file = file
        .strip_prefix(root)
        .whatever_context(format!("Could not strip \"{:?}\" of \"{:?}\"", file, root))?;

    let resource_type = relative_file.iter().next().unwrap();
    let resource = relative_file.strip_prefix(resource_type).unwrap().to_string_lossy();

    println!("type: {:?}, resource: {:?}", resource_type, resource);

    match resource_type.to_string_lossy().as_ref() {
        "ranks" => {
            let ranks = &mut global_state.resources.write().expect("not poisoned").ranks;

            if ranks.get_index(resource.as_ref()).is_some() {
                ranks.remove_resource(resource.as_ref()).unwrap();

                let rank = load_rank_from_file(file)?;

                ranks.add_resource(resource.into_owned(), rank);
            }
        }
        "samples" => {}
        _ => {}
    }

    Ok(())
}

pub fn load(
    path: &Path,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<mpsc::Receiver<Result<Event, Error>>, EngineError> {
    let parent = path
        .parent()
        .whatever_context(format!("path {:?} has no parent", path))?;

    let json_raw = fs::read_to_string(path).context(IoSnafu)?;
    let json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    if let Some(version) = json["version"].as_str() {
        if version != VERSION.to_string() {
            migrate(PathBuf::from(path))?;
        }
    }

    let json_raw = fs::read_to_string(path).context(IoSnafu)?;
    let mut json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    *state = NodeState::new(global_state).unwrap();
    global_state.reset();

    let mut resources = global_state.resources.write().unwrap();

    load_resources(
        &parent.join("samples"),
        &mut resources.samples,
        AUDIO_EXTENSIONS,
        &load_sample,
    )?;
    load_resources(
        &parent.join("ranks"),
        &mut resources.ranks,
        &["toml"],
        &load_rank_from_file,
    )?;

    let json_state = &mut json["state"];

    let graph_manager = serde_json::from_value(json_state["graph_manager"].take()).context(JsonParserSnafu)?;
    let root_graph_index = serde_json::from_value(json_state["root_graph_index"].take()).context(JsonParserSnafu)?;
    let output_node = serde_json::from_value(json_state["output_node"].take()).context(JsonParserSnafu)?;
    let midi_in_node = serde_json::from_value(json_state["midi_in_node"].take()).context(JsonParserSnafu)?;
    let socket_registry = serde_json::from_value(json_state["socket_registry"].take()).context(JsonParserSnafu)?;

    state.load_state(
        graph_manager,
        root_graph_index,
        output_node,
        midi_in_node,
        socket_registry,
    );

    let (tx, rx) = mpsc::channel();
    let mut watcher =
        RecommendedWatcher::new(tx, Config::default()).whatever_context("Could not create file watcher")?;

    watcher
        .watch(parent.as_ref(), RecursiveMode::Recursive)
        .whatever_context("Could not create file watcher")?;

    Ok(rx)
}
