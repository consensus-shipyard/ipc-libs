use std::collections::{HashMap, HashSet};

use ipc_sdk::subnet_id::SubnetID;
use libp2p::PeerId;

use crate::provider_record::{ProviderRecord, Timestamp};

pub struct SubnetProviderCache {
    /// Maximum number of subnets to track, to protect against DoS attacks, trying to
    /// flood someone with subnets that don't actually exist. When the number of subnets
    /// reaches this value, we remove the subnet with the smallest number of providers;
    /// hopefully this would be a subnet
    max_subnets: usize,
    /// Set of peers with known addresses. Only such peers can be added to the cache.
    routable_peers: HashSet<PeerId>,
    /// List of peer IDs supporting each subnet.
    subnet_providers: HashMap<SubnetID, HashSet<PeerId>>,
    /// Timestamp of the last record received about a peer.
    peer_timestamps: HashMap<PeerId, Timestamp>,
}

impl SubnetProviderCache {
    pub fn new(max_subnets: usize) -> Self {
        Self {
            max_subnets,
            routable_peers: Default::default(),
            subnet_providers: Default::default(),
            peer_timestamps: Default::default(),
        }
    }

    /// Mark a peer as routable.
    ///
    /// Once routable, the cache will keep track of provided subnets.
    pub fn set_routable(&mut self, peer_id: PeerId) {
        self.routable_peers.insert(peer_id);
    }

    /// Mark a previously routable peer as unroutable.
    ///
    /// Once unroutable, the cache will stop tracking the provided subnets.
    pub fn set_unroutable(&mut self, peer_id: PeerId) {
        self.routable_peers.remove(&peer_id);
        self.peer_timestamps.remove(&peer_id);
        for providers in self.subnet_providers.values_mut() {
            providers.remove(&peer_id);
        }
    }

    /// Check if a peer has been marked as routable.
    pub fn is_routable(&self, peer_id: PeerId) -> bool {
        self.routable_peers.contains(&peer_id)
    }

    /// Try to add a provider to the cache.
    ///
    /// Return `true` if succeeded, `false` if the peer is not yet routable.
    pub fn add_provider(&mut self, record: ProviderRecord) -> bool {
        if !self.is_routable(record.peer_id) {
            return false;
        }

        let timestamp = self
            .peer_timestamps
            .entry(record.peer_id)
            .or_insert(record.timestamp);

        if *timestamp < record.timestamp {
            *timestamp = record.timestamp;
            for subnet_id in record.subnet_ids {
                let providers = self.subnet_providers.entry(subnet_id).or_default();
                providers.insert(record.peer_id);
            }
            self.prune_subnets();
        }

        true
    }

    /// Ensure we don't have more than `max_subnets` number of subnets in the cache.
    fn prune_subnets(&mut self) {
        let to_prune = self.subnet_providers.len().saturating_sub(self.max_subnets);

        if to_prune > 0 {
            let mut counts = self
                .subnet_providers
                .iter()
                .map(|(id, ps)| (id.clone(), ps.len()))
                .collect::<Vec<_>>();

            counts.sort_by_key(|(_, count)| *count);

            for (subnet_id, _) in counts.into_iter().take(to_prune) {
                self.subnet_providers.remove(&subnet_id);
            }
        }
    }

    /// Prune any provider which hasn't provided an update since a cutoff timestamp.
    pub fn prune_providers(&mut self, cutoff_timestamp: Timestamp) {
        let to_prune = self
            .peer_timestamps
            .iter()
            .filter_map(|(id, ts)| {
                if *ts < cutoff_timestamp {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for peer_id in to_prune {
            self.set_unroutable(peer_id)
        }
    }

    /// List any known providers of a subnet.
    pub fn providers_of_subnet(&self, subnet_id: &SubnetID) -> Vec<PeerId> {
        self.subnet_providers
            .get(subnet_id)
            .map(|hs| hs.iter().cloned().collect())
            .unwrap_or_default()
    }
}
