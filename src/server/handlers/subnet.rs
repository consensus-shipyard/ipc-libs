use std::collections::HashMap;
use once_cell::sync::OnceCell;
use crate::config::Subnet;
use crate::jsonrpc::JsonRpcClientImpl;
use crate::lotus::client::LotusJsonRPCClient;
use crate::manager::LotusSubnetManager;

/// The json rpc subnet manager wrapper struct. This struct can be shared by all the subnet methods.
/// In this case, there is no need to re-init the same SubnetManager for different methods.
pub(crate) struct SubnetManagerShared {
    subnets: HashMap<String, Subnet>,
    manager: HashMap<String, OnceCell<LotusSubnetManager<JsonRpcClientImpl>>>,
}

impl SubnetManagerShared {
    pub fn new(subnets: HashMap<String, Subnet>) -> Self {
        let mut manager = HashMap::new();
        subnets.keys().for_each(|subnet| {
            manager.insert(subnet.clone(), OnceCell::new());
        });
        Self { subnets, manager }
    }

    pub fn get_subnet(&self, subnet_str: &String) -> Option<&Subnet> {
        self.subnets.get(subnet_str)
    }

    pub fn get_manager_and_gateway(&self, subnet_str: &String) -> Option<(&LotusSubnetManager<JsonRpcClientImpl>, u64)> {
        if !self.subnets.contains_key(subnet_str) {
            return None;
        }

        let subnet = self.subnets.get(subnet_str).unwrap();
        let manager_cell = self.manager.get(subnet_str).unwrap();
        let manager = manager_cell.get_or_init(|| {
            let json_rpc_client = JsonRpcClientImpl::new(
                subnet.jsonrpc_api_http.clone(),
                subnet.auth_token.clone().as_deref(),
            );
            let lotus_client = LotusJsonRPCClient::new(json_rpc_client);
            LotusSubnetManager::new(lotus_client)
        });

        Some((manager, subnet.gateway_addr))
    }
}
