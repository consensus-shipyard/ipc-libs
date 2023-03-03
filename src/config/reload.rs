// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Reloadable config

use crate::config::Config;
use anyhow::{anyhow, Result};
use std::ops::DerefMut;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::sync::{broadcast, mpsc, oneshot};

/// Internal request for reloadable config
#[derive(Debug)]
enum Request {
    Reload {
        response_tx: oneshot::Sender<Result<()>>,
        path: String,
    },
    Stop,
}

/// Reloadable configuration.
pub struct ReloadableConfig {
    config: Arc<RwLock<Arc<Config>>>,
    request_tx: mpsc::Sender<Request>,

    broadcast_tx: broadcast::Sender<()>,
    #[allow(dead_code)]
    broadcast_rx: broadcast::Receiver<()>,
}

impl ReloadableConfig {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        // we dont really need a big channel, the frequency should be very very low
        let channel_size = 8;
        let (request_tx, mut request_rx) = mpsc::channel(channel_size);
        let (broadcast_tx, broadcast_rx) = broadcast::channel(channel_size);

        let config = Arc::new(RwLock::new(Arc::new(Config::from_file(path)?)));
        let config_cloned = config.clone();
        let broadcast_tx_cloned = broadcast_tx.clone();
        tokio::spawn(async move {
            while let Some(request) = request_rx.recv().await {
                match request {
                    Request::Reload { response_tx, path } => {
                        let c = match Config::from_file_async(path).await {
                            Ok(c) => c,
                            Err(e) => {
                                log::error!("cannot reload config due to: {e:?}");
                                // we dont really care about if the other side is still listening
                                response_tx
                                    .send(Err(anyhow!("cannot reload config")))
                                    .unwrap_or_default();
                                continue;
                            }
                        };

                        let mut config = config_cloned.write().unwrap();
                        let r = config.deref_mut();
                        *r = Arc::new(c);

                        response_tx.send(Ok(())).unwrap_or_default();
                        broadcast_tx_cloned.send(()).unwrap_or_default();
                    }
                    Request::Stop => {
                        break;
                    }
                }
            }
            log::info!("config reloading exited");
        });

        Ok(Self {
            config,
            request_tx,
            broadcast_tx,
            broadcast_rx,
        })
    }

    /// Read from the config file.
    pub fn get_config(&self) -> Arc<Config> {
        let config = self.config.read().unwrap();
        config.clone()
    }

    /// Triggers a reload of the config from the target path
    pub async fn reload(&self, path: String) -> Result<()> {
        let (response_tx, rx) = oneshot::channel();

        self.request_tx
            .send(Request::Reload { response_tx, path })
            .await?;

        rx.await?
    }

    /// Stops the reloading of config. Once this is called, `reload` will no longer trigger reloading.
    pub async fn stop_reloading(&self) -> Result<()> {
        let request = Request::Stop;
        self.request_tx.send(request).await?;
        Ok(())
    }

    pub fn new_subscriber(&self) -> broadcast::Receiver<()> {
        self.broadcast_tx.subscribe()
    }
}
