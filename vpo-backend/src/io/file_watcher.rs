use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use futures::channel::mpsc::{channel, Receiver};
use notify::{Error, RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use snafu::ResultExt;

use crate::errors::EngineError;

pub struct FileWatcher {
    debouncer: Debouncer<RecommendedWatcher>,
    path: Option<PathBuf>,
}

impl FileWatcher {
    pub fn new() -> Result<(FileWatcher, Receiver<Result<Vec<DebouncedEvent>, Vec<Error>>>), EngineError> {
        // many events may be produced during a file import, the channel needs to be ready for that
        let (mut tx, rx) = channel(1024);

        let debouncer = new_debouncer(Duration::from_millis(100), None, move |res| {
            tx.try_send(res).unwrap();
        })
        .whatever_context("Could not create file watcher")?;

        Ok((FileWatcher { debouncer, path: None }, rx))
    }

    pub fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<(), EngineError> {
        if let Some(path) = &self.path {
            self.debouncer
                .watcher()
                .unwatch(&path)
                .whatever_context("Could not unwatch")?;
        }

        self.debouncer
            .watcher()
            .watch(path.as_ref(), RecursiveMode::Recursive)
            .whatever_context("Could not watch path")?;
        self.path = Some(path.as_ref().into());

        Ok(())
    }
}
