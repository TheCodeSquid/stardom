use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode, Watcher as _},
    DebounceEventResult, FileIdMap,
};
use tokio::sync::broadcast;

type Debouncer = notify_debouncer_full::Debouncer<RecommendedWatcher, FileIdMap>;

#[derive(Clone)]
pub struct Watcher {
    reload: broadcast::Sender<()>,
    building: Arc<AtomicBool>,
    _debouncer: Arc<Option<Debouncer>>,
}

impl Watcher {
    pub fn off() -> Self {
        Self {
            reload: broadcast::channel(1).0,
            building: Arc::default(),
            _debouncer: Arc::default(),
        }
    }

    pub fn watch<P, F>(path: P, mut track: F) -> Result<Self>
    where
        P: AsRef<Path>,
        F: FnMut(PathBuf) -> bool + Send + 'static,
    {
        let reload = broadcast::channel(1).0;

        let reload_tx = reload.clone();
        let mut debouncer = new_debouncer(
            Duration::from_millis(200),
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => {
                    for path in events.into_iter().flat_map(|event| event.event.paths) {
                        if track(path) {
                            let _ = reload_tx.send(());
                            break;
                        }
                    }
                }
                Err(_) => todo!(),
            },
        )?;

        debouncer
            .watcher()
            .watch(path.as_ref(), RecursiveMode::Recursive)?;

        Ok(Self {
            reload,
            building: Arc::default(),
            _debouncer: Arc::new(Some(debouncer)),
        })
    }

    pub fn build_lock(&self) -> BuildGuard {
        self.building.store(true, Ordering::Release);
        BuildGuard(self.building.clone())
    }

    pub async fn recv(&self) -> Result<(), broadcast::error::RecvError> {
        let mut reload = self.reload.subscribe();
        loop {
            reload.recv().await?;
            if !self.building.load(Ordering::Acquire) {
                break;
            }
        }
        Ok(())
    }
}

pub struct BuildGuard(Arc<AtomicBool>);

impl Drop for BuildGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
