// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Hot reloading config

use crate::config::Config;
use anyhow::{anyhow, Result};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, RecvTimeoutError, TryRecvError};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;

pub struct HotReloadingConfig {
    config: Arc<RwLock<ConfigWrapper>>,
    stop_signal_tx: mpsc::Sender<()>,
    reload_notify_rx: mpsc::Receiver<()>,
}

impl HotReloadingConfig {
    pub fn new_with_watcher(path: String, watch_interval: u64) -> Result<Self> {
        let (stop_signal_tx, stop_signal_rx) = channel();
        let (reload_notify_tx, reload_notify_rx) = channel();

        let config = Arc::new(RwLock::new(ConfigWrapper {
            config: Config::from_file(path.clone())?,
        }));

        let config_cloned = config.clone();
        thread::spawn(move || {
            match watch_and_update(
                &config_cloned,
                &path,
                stop_signal_rx,
                watch_interval,
                reload_notify_tx,
            ) {
                Ok(_) => {}
                Err(e) => {
                    println!("watch log failed due to {e:?}");
                    log::error!("watch log failed due to {e:?}");
                }
            }
        });

        Ok(Self {
            config,
            stop_signal_tx,
            reload_notify_rx,
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

    /// Attempts to get a modification signal without blocking. Returns true if modification is
    /// observed, returns false if no modification is observed.
    pub fn try_wait_modification(&self) -> Result<bool> {
        let r = match self.reload_notify_rx.try_recv() {
            Ok(_) => true,
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => return Err(anyhow!("notification stopped")),
        };
        Ok(r)
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
    reload_notify_tx: mpsc::Sender<()>,
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

    loop {
        if let Ok(()) = stop_signal_rx.try_recv() {
            break;
        }

        let event = match rx.recv_timeout(Duration::from_secs(watch_interval)) {
            Ok(event) => event?,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        if let Event {
            kind: notify::event::EventKind::Modify(_),
            ..
        } = event
        {
            match Config::from_file(path.as_ref()) {
                Ok(c) => {
                    config.write().unwrap().config = c;
                    reload_notify_tx.send(())?;
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
