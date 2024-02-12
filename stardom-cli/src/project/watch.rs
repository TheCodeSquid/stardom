use std::{path::Path, time::Duration};

use anyhow::Result;
use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode, Watcher as _},
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap,
};
use tokio::sync::{broadcast, mpsc};

use crate::shell;

pub struct Watcher {
    _debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
    recv: mpsc::UnboundedReceiver<Vec<DebouncedEvent>>,
    abort: broadcast::Sender<()>,
}

impl Watcher {
    pub async fn recv(&mut self) -> Option<Vec<DebouncedEvent>> {
        self.recv.recv().await
    }

    pub fn abort_sender(&self) -> broadcast::Sender<()> {
        self.abort.clone()
    }
}

pub async fn watcher<P: AsRef<Path>>(path: P) -> Result<Watcher> {
    let (tx, rx) = mpsc::unbounded_channel();

    let mut debouncer = new_debouncer(
        Duration::from_millis(250),
        None,
        move |result: DebounceEventResult| match result {
            Ok(events) => {
                let _ = tx.send(events);
            }
            Err(errors) => errors.into_iter().for_each(|error| {
                shell().error(error);
            }),
        },
    )?;

    let path = path.as_ref();
    debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
    shell().status("Watching", path.display());

    let (abort, _) = broadcast::channel(1);
    Ok(Watcher {
        _debouncer: debouncer,
        recv: rx,
        abort,
    })
}
