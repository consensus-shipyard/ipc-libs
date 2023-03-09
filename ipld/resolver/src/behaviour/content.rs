// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    task::{Context, Poll},
    time::Duration,
};

use libipld::{store::StoreParams, Cid};
use libp2p::{
    core::ConnectedPoint,
    request_response::handler::RequestResponseHandlerEvent,
    swarm::{
        derive_prelude::{ConnectionId, FromSwarm},
        ConnectionHandler, IntoConnectionHandler, NetworkBehaviour, NetworkBehaviourAction,
        PollParameters,
    },
    Multiaddr, PeerId,
};
use libp2p_bitswap::{Bitswap, BitswapConfig, BitswapEvent, BitswapStore};
use log::warn;
use prometheus::Registry;

use crate::{
    limiter::{RateLimit, RateLimiter},
    stats,
};

pub type QueryId = libp2p_bitswap::QueryId;

// Not much to do here, just hiding the `Progress` event as I don't think we'll need it.
// We can't really turn it into anything more meaningful; the outer Service, which drives
// the Swarm events, will have to store the `QueryId` and figure out which CID it was about
// (there could be multiple queries running over the same CID) and how to respond to the
// original requestor (e.g. by completing a channel).
#[derive(Debug)]
pub enum Event {
    /// Event raised when a resolution request is finished.
    ///
    /// The result will indicate either success, or arbitrary failure.
    /// If it is a success, the CID can be found in the [`BitswapStore`]
    /// instance the behaviour was created with.
    ///
    /// Note that it is possible that the synchronization completed
    /// partially, but some recursive constituent is missing. The
    /// caller can use the [`missing_blocks`] function to check
    /// whether a retry is necessary.
    Complete(QueryId, anyhow::Result<()>),
}

/// Configuration for [`content::Behaviour`].
#[derive(Debug, Clone)]
pub struct Config {
    /// Number of bytes that can be consumed remote peers in a time period.
    ///
    /// 0 means no limit.
    pub rate_limit_bytes: u32,
    /// Length of the time period at which the consumption limit fills.
    ///
    /// 0 means no limit.
    pub rate_limit_period: Duration,
}

/// Behaviour built on [`Bitswap`] to resolve IPLD content from [`Cid`] to raw bytes.
pub struct Behaviour<P: StoreParams> {
    inner: Bitswap<P>,
    /// Remember which address peers connected from, so we can apply the rate limit
    /// on the address, and not on the peer ID which they can change easily.
    peer_addresses: HashMap<PeerId, Multiaddr>,
    /// Limit the amount of data served by remote address.
    rate_limiter: RateLimiter<Multiaddr>,
    rate_limit: RateLimit,
}

impl<P: StoreParams> Behaviour<P> {
    pub fn new<S>(config: Config, store: S) -> Self
    where
        S: BitswapStore<Params = P>,
    {
        let bitswap = Bitswap::new(BitswapConfig::default(), store);
        Self {
            inner: bitswap,
            peer_addresses: Default::default(),
            rate_limiter: RateLimiter::new(config.rate_limit_period),
            rate_limit: RateLimit::new(config.rate_limit_bytes, config.rate_limit_period),
        }
    }

    /// Register Prometheus metrics.
    pub fn register_metrics(&self, registry: &Registry) -> anyhow::Result<()> {
        self.inner.register_metrics(registry)
    }

    /// Recursively resolve a [`Cid`] and all underlying CIDs into blocks.
    ///
    /// The [`Bitswap`] behaviour will call the [`BitswapStore`] to ask for
    /// blocks which are missing, ie. find CIDs which aren't available locally.
    /// It is up to the store implementation to decide which links need to be
    /// followed.
    ///
    /// It is also up to the store implementation to decide which CIDs requests
    /// to responds to, e.g. if we only want to resolve certain type of content,
    /// then the store can look up in a restricted collection, rather than the
    /// full IPLD store.
    ///
    /// Resolution will be attempted from the peers passed to the method,
    /// starting with the first one with `WANT-BLOCK`, then whoever responds
    /// positively to `WANT-HAVE` requests. The caller should talk to the
    /// `membership::Behaviour` first to find suitable peers, and then
    /// prioritise peers which are connected.
    ///
    /// The underlying [`libp2p_request_response::RequestResponse`] behaviour
    /// will initiate connections to the peers which aren't connected at the moment.
    pub fn resolve(&mut self, cid: Cid, peers: Vec<PeerId>) -> QueryId {
        stats::CONTENT_RESOLVE_RUNNING.inc();
        // Not passing any missing items, which will result in a call to `BitswapStore::missing_blocks`.
        self.inner.sync(cid, peers, [].into_iter())
    }

    /// Check if we are using rate limiting.
    fn has_rate_limits(&self) -> bool {
        !(self.rate_limit.resource_limit == 0 || self.rate_limit.period.is_zero())
    }

    /// Check whether the peer has already exhaused their rate limit.
    fn check_rate_limit(&mut self, peer_id: &PeerId, cid: &Cid) -> bool {
        if !self.has_rate_limits() {
            return true;
        }
        if let Some(addr) = self.peer_addresses.get(peer_id).cloned() {
            let bytes = cid.to_bytes().len().try_into().unwrap_or(u32::MAX);

            if !self.rate_limiter.add(&self.rate_limit, addr, bytes) {
                return false;
            }
        }
        true
    }
}

impl<P: StoreParams> NetworkBehaviour for Behaviour<P> {
    type ConnectionHandler = <Bitswap<P> as NetworkBehaviour>::ConnectionHandler;
    type OutEvent = Event;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        self.inner.new_handler()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.inner.addresses_of_peer(peer_id)
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        // Store the remote address.
        match &event {
            FromSwarm::ConnectionEstablished(c) => {
                if c.other_established == 0 {
                    let peer_addr = match c.endpoint {
                        ConnectedPoint::Dialer {
                            address: remote_addr,
                            ..
                        } => remote_addr,
                        ConnectedPoint::Listener {
                            send_back_addr: remote_addr,
                            ..
                        } => remote_addr,
                    };
                    self.peer_addresses.insert(c.peer_id, peer_addr.clone());
                }
            }
            FromSwarm::ConnectionClosed(c) => {
                if c.remaining_established == 0 {
                    self.peer_addresses.remove(&c.peer_id);
                }
            }
            _ => {}
        }

        self.inner.on_swarm_event(event)
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: <<Self::ConnectionHandler as IntoConnectionHandler>::Handler as ConnectionHandler>::OutEvent,
    ) {
        if let RequestResponseHandlerEvent::Request { request, .. } = &event {
            if !self.check_rate_limit(&peer_id, &request.cid) {
                warn!("rate limiting {peer_id}");
                stats::CONTENT_RATE_LIMITED.inc();
                return;
            }
        }

        self.inner
            .on_connection_handler_event(peer_id, connection_id, event)
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        params: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        while let Poll::Ready(ev) = self.inner.poll(cx, params) {
            match ev {
                NetworkBehaviourAction::GenerateEvent(ev) => match ev {
                    BitswapEvent::Progress(_, _) => {}
                    BitswapEvent::Complete(id, result) => {
                        stats::CONTENT_RESOLVE_RUNNING.dec();
                        let out = Event::Complete(id, result);
                        return Poll::Ready(NetworkBehaviourAction::GenerateEvent(out));
                    }
                },
                other => {
                    return Poll::Ready(other.map_out(|_| unreachable!("already handled")));
                }
            }
        }

        Poll::Pending
    }
}
