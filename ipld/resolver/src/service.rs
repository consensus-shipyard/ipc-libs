use libipld::store::StoreParams;
use libp2p::Swarm;

use crate::behaviour::IpldResolver;

#[allow(dead_code)] // TODO (IPC-37): Remove this.
pub struct IpldResolverService<P: StoreParams> {
    swarm: Swarm<IpldResolver<P>>,
}

impl<P: StoreParams> IpldResolverService<P> {
    /// Start the swarm listening for incoming connections and drive the events forward.
    #[allow(dead_code)] // TODO (IPC-37): Remove this.
    pub async fn run(self) -> anyhow::Result<()> {
        todo!("IPC-37")
    }
}
