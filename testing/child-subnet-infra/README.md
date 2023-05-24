# Child Subnet Infra 
This is a utility script to spawn child subnet infrastructure, mainly for integration testing. 

It spawns the required number of eudico nodes and validators based on input config parameters. This would be a time saver 
when testing subnet checkpoints and cross net messages.

The`src/main.rs` contains the configuration for the subnet topology and binary path definition.
The `src/infra` contains the process flow for spawn subnet nodes and their validators.

To test this, one must first manually start the root net and the ipc agent. Then start the process with:
```shell
cargo build --release
../target/release/child-subnet-infra
```