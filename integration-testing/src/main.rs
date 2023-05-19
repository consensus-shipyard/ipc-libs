use ipc_sdk::subnet_id::SubnetID;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicU16;

mod infra;

#[tokio::main]
async fn main() {
    let api_port_sequence = Arc::new(AtomicU16::new(5));
    let topology = infra::SubnetTopology::new(
        SubnetID::from_str("/root/t01002").unwrap(),
        "test-subnet-1".to_string(),
        "t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq".to_string(),
        2,
        "~/lotus/eudico".to_string(),
        Some(SubnetID::from_str("/root").unwrap()),
        api_port_sequence,
    );

    let r = infra::subnet::spawn_child_subnet(&topology).await;
    if r.is_err() {
        log::error!("cannot spawn subnet: {e:}");
    }
}
