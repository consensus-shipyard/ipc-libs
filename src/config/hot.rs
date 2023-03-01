//! Hot reloading config

use crate::config::Config;
use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;

pub struct HotReloadingConfig {
    config: Arc<RwLock<ConfigWrapper>>,
    stop_signal_tx: mpsc::Sender<()>,
}

impl HotReloadingConfig {
    pub fn new_with_watcher(path: String, watch_interval: u64) -> Result<Self> {
        let (stop_signal_tx, stop_signal_rx) = channel();

        let config = Arc::new(RwLock::new(ConfigWrapper {
            config: Config::from_file(path.clone())?,
        }));

        let config_cloned = config.clone();
        thread::spawn(move || {
            match watch_and_update(&config_cloned, &path, stop_signal_rx, watch_interval) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("watch log failed due to {e:?}");
                }
            }
        });

        Ok(Self {
            config,
            stop_signal_tx,
        })
    }

    /// Read from the config file. Since `Config
    pub fn read_from_config(&self, mut read_fn: impl FnMut(&Config)) {
        let config = self.config.read().unwrap();
        read_fn(&config.config);
    }

    pub fn stop_watching(&self) {
        self.stop_signal_tx.send(()).unwrap_or_default();
    }
}

struct ConfigWrapper {
    config: Config,
}

fn watch_and_update(
    config: &Arc<RwLock<ConfigWrapper>>,
    path: impl AsRef<Path>,
    stop_signal_rx: mpsc::Receiver<()>,
    watch_interval: u64,
) -> Result<()> {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher: RecommendedWatcher = Watcher::new(
        tx,
        notify::Config::default().with_poll_interval(Duration::from_secs(watch_interval)),
    )?;

    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    // This is a simple loop, but you may want to use more complex logic here,
    // for example to handle I/O.
    loop {
        if let Ok(()) = stop_signal_rx.try_recv() {
            break;
        }

        let event = rx.recv_timeout(Duration::from_secs(watch_interval))??;
        if let Event {
            kind: notify::event::EventKind::Modify(_),
            ..
        } = event
        {
            match Config::from_file(path.as_ref()) {
                Ok(c) => {
                    config.write().unwrap().config = c;
                }
                Err(e) => {
                    log::error!("cannot read config file: {e:?}");
                }
            }
        }
    }

    log::info!("log watcher terminated");
    Ok(())
}
