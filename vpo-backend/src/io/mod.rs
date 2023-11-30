pub mod cpal;
pub mod file_watcher;
pub mod midir;
mod scoped_pool;

use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Instant;

use lazy_static::lazy_static;

use common::resource_manager::ResourceManager;
use node_engine::global_state::Resources;
use node_engine::{global_state::GlobalState, state::GraphState};
use notify::{Config, Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use semver::Version;
use serde_json::{json, Value};
use snafu::{OptionExt, ResultExt};
use walkdir::WalkDir;

use crate::errors::{IoSnafu, JsonParserSnafu};

use crate::errors::EngineError;
use crate::migrations::migrate;
use crate::resource::rank::load_rank_from_file;
use crate::resource::sample::load_sample;
use crate::resource::ui::load_ui_from_file;

use self::scoped_pool::scoped_pool;

const AUDIO_EXTENSIONS: &[&str] = &["ogg", "wav", "mp3", "flac"];
lazy_static! {
    pub static ref VERSION: Version = Version::parse("0.4.0").unwrap();
}

pub fn save(state: &GraphState, path: &Path) -> Result<(), EngineError> {
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
    valid_extensions: &[&str],
    load_resource: &F,
) -> Result<ResourceManager<T>, EngineError>
where
    T: Send + Sync + Debug,
    F: Fn(&Path) -> Result<T, EngineError> + Send + Sync,
{
    let mut resources = ResourceManager::new();

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
    let new_resources = scoped_pool(asset_list, &|(key, location)| {
        let new_resource = load_resource(&location).unwrap();

        (key, new_resource)
    })
    .unwrap();

    for (key, resource) in new_resources.into_iter() {
        resources.add_resource(key, resource);
    }

    Ok(resources)
}

/// Make sure samples are loaded before ranks!
pub fn load_single(root: &Path, file: &Path, resources: &mut Resources) -> Result<(), EngineError> {
    let relative_file = file
        .strip_prefix(root)
        .whatever_context(format!("Could not strip \"{:?}\" of \"{:?}\"", file, root))?;

    let resource_type = relative_file.iter().next().unwrap();
    let resource = relative_file.strip_prefix(resource_type).unwrap().to_string_lossy();

    println!("type: {:?}, resource: {:?}", resource_type, resource);

    match resource_type.to_string_lossy().as_ref() {
        "ranks" => {
            let mut resources = resources;

            if resources.ranks.get_index(resource.as_ref()).is_some() {
                resources.ranks.remove_resource(resource.as_ref());
            }

            let rank = load_rank_from_file(file, &resources.samples)?;
            resources.ranks.add_resource(resource.into_owned(), rank);
        }
        "samples" => {
            if resources.samples.get_index(resource.as_ref()).is_some() {
                resources.samples.remove_resource(resource.as_ref());
            }

            let sample = load_sample(file)?;
            resources.samples.add_resource(resource.into_owned(), sample);
        }
        "ui" => {
            if resources.ui.get_index(resource.as_ref()).is_some() {
                resources.ui.remove_resource(resource.as_ref());
            }

            let ui_element = load_ui_from_file(file)?;
            resources.ui.add_resource(resource.into_owned(), ui_element);
        }
        _ => {}
    }

    Ok(())
}

pub fn load(
    path: &Path,
    state: &mut GraphState,
    global_state: &mut GlobalState,
    resources: &mut Resources,
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

    *state = GraphState::new(global_state).unwrap();
    resources.reset();

    println!("Loading resources...");
    let time = Instant::now();

    let samples = load_resources(&parent.join("samples"), AUDIO_EXTENSIONS, &load_sample)?;
    let ranks = load_resources(&parent.join("ranks"), &["toml"], &|path| {
        load_rank_from_file(path, &samples)
    })?;
    let ui = load_resources(&parent.join("ui"), &["toml"], &load_ui_from_file)?;

    resources.samples.extend(samples);
    resources.ranks.extend(ranks);
    resources.ui.extend(ui);

    println!("Loaded! Took {:?} seconds", time.elapsed());

    let json_state = &mut json["state"];

    let graph_manager = serde_json::from_value(json_state["graphManager"].take()).context(JsonParserSnafu)?;
    let root_graph_index = serde_json::from_value(json_state["rootGraphIndex"].take()).context(JsonParserSnafu)?;
    let io_nodes = serde_json::from_value(json_state["ioNodes"].take()).context(JsonParserSnafu)?;

    state.load_state(graph_manager, root_graph_index, io_nodes);

    let (tx, rx) = mpsc::channel();
    let mut watcher =
        RecommendedWatcher::new(tx, Config::default()).whatever_context("Could not create file watcher")?;

    watcher
        .watch(parent.as_ref(), RecursiveMode::Recursive)
        .whatever_context("Could not create file watcher")?;

    Ok(rx)
}
