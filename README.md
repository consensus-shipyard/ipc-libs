# IPC Agent
> The InterPlanetary Consensus (IPC) orchestrator.

The IPC Agent is the entrypoint to interacting with IPC. It is a client application that provides a simple and easy-to-use interface to interact with IPC as a user and run all the processes required for the operation of a subnet.

See the [docs](docs) for a conceptual overview.
a
## Installation
### Build requirements
To build the IPC Agent you need to have Rust installed in your environment. The current MSRV (Minimum Supported Rust Version) is nightly-2022-10-03 due to some test build dependencies. A working version is tracked in rust-toolchain (this is picked up by rustup automatically). You can look for instructions on [how to run Rust and rustup following this link](https://www.rust-lang.org/tools/install).

### Build instructions
To build the binary for the IPC agent you need to build the requirements in your environment, clone this repo, and build the binary following these steps:
```bash
$ git clone https://github.com/consensus-shipyard/ipc-agent.git
$ cd ipc-agent
$ rustup target add wasm32-unknown-unknown
$ make build
```

This builds the binary of the IPC agent in the `./bin` folder of your repo. If you want to make the command available everywhere, add this folder to the binary `PATH` of your system. To see if the installation was successfully you can run the following command: 
``` bash
$ ./bin/ipc_agent --help

The IPC agent command line tool

Usage: ipc_agent [OPTIONS] <COMMAND>

Commands:
  daemon                Launch the ipc agent daemon
  reload-config         Config commands
  init-config           Arguments to initialize a new empty config file
  create-subnet         Subnet manager commands
  list-subnets          List child subnets
  join-subnet           Join a subnet
  leave-subnet          Leaving a subnet
  kill-subnet           Kill an existing subnet
  fund                  Send funds from a parent to a child subnet
  release               Release operation in the gateway actor
  propagate             Propagate operation in the gateway actor
  whitelist-propagator  Whitelist propagators in the gateway actor
  send-value            Send value to an address within a subnet
  wallet-new            Create new wallet in subnet
  help                  Print this message or the help of the given subcommand(s)

Options:
  -c, --config-path <CONFIG_PATH>  The toml config file path for IPC Agent, default to ${HOME}/.ipc_agent/config.toml
  -h, --help                       Print help
  -V, --version                    Print version
```

## Infrastructure
IPC currently uses [a fork of Lotus](https://github.com/consensus-shipyard/lotus), that we like to call _Eudico_, to run its subnets. The IPC agent does nothing by itself, and is just an orchestrator over existing subnet deployments. To ease the deployment of new subnets and nodes, we provide a set of convenient scripts to deploy all the infrastructure required to run IPC. 

### Install infrastructure scripts
[Eudico](https://github.com/consensus-shipyard/lotus/tree/spacenet/scripts/ipc) provides a set of infrastructure scripts, which assume a working installation of Docker. To install Docker [follow this link])(https://docs.docker.com/get-docker/) and choose your working environment.

With Docker installed, you can then `make install-infra` in this repository to clone the eudico repo, build the docker image that you need to run subnets, and install the infrastructure scripts in the `./bin` folder.

To check if the installation was successful you can run the following command, and it should return a similar output: 
```bash
$ docker images | grep eudico
eudico                      latest        8fb6db609712   2 minutes ago   341MB
```

## Usage
### Configuration
If you are running the agent for the first time, the first thing you need to do is to create a new config. The default config path for the agent is `~/.ipc_agent/config.toml`. The agent will always try to pick up the config from this path unless told otherwise. To populate a sample config file in the default path, you can run the following command:
```bash
$ ./bin/ipc_agent init-config
```
If you `cat ~/.ipc_agent/config.toml` you should see a new config populated with a sample root and subnet configurations.

### Running the daemon
The IPC agent runs as a foreground daemon process that spawns a new JSON RPC server to interact with it, and all the processes to automatically handle checkpoints and the execution of cross-net messages for the subnets our agent is participating in. The agent determines the list of subnets it should interact with from its config file. 

Alternatively, the agent can also be used as a CLI to interact with IPC. Under the hood, this cli sends new commands to the RPC server of the daemon. To run the IPC agent daemon you can run:
```
$ ./bin/ipc_agent daemon
```
The RPC server of the daemon will be listening to the endpoint determined in the `json_rpc_address` field of the config. If you are looking for your agent to be accessible from Docker or externally, remember to listen from `0.0.0.0` instead of `127.0.0.1` as specified in the empty configuration. 

### Interacting with a rootnet
#### Spacenet
> WIP: This instructions will be updated once IPC has been fully deployed in Spacenet.

#### Local deployment
To deploy sample rootnet locally for testing you can use the IPC scripts installed in `./bin/ipc-infra` (refer to the [installation of infrastructure](#Installation-infrastructure-scripts)) by running:
```bash
$ ./bin/ipc-infra/run-root-docker-1val.sh <lotus-api-port> <validator-libp2p-port>
```

For instance, running `./bin/ipc-infra/run-root-docker-1val.sh 1235 1379` will run a rootnet daemon listening at `localhost:1235`, and a single validator mining in the rootnet listening through its libp2p host in `localhost:1379`. The end of the log in the execution of this script should look something like: 
```
>>> Root daemon running in container: 84711d67cf162e30747c4525d69728c4dea8c6b4b35cd89f6d0947fee14bf908
>>> Token to /root daemon: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.j94YYOr8_AWhGGHQd0q8JuQVuNhJA017SK9EUkqDOO0
>>> Default wallet: t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq
```
This information will be relevant to configure our agent to connect to this rootnet node. For this, the `config.toml` section should be updated accordingly. In the above case, we need to set the endpoint of our rootnet node to be `127.0.0.1:1235`, we need to set `auth_token` to the one provided by the script, and the default account, for instance, the one provided by the script (although we could use ant other).

The configuration for our rootnet should look therefore like this:
```toml
[subnets."/root"]
id = "/root"
jsonrpc_api_http = "http://127.0.0.1:1235/rpc/v1"
jsonrpc_api_ws = "wss://example.org/rpc/v0"
auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.j94YYOr8_AWhGGHQd0q8JuQVuNhJA017SK9EUkqDOO0"
accounts = ["t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq"]
```

> Beware: If you are already running the daemon, changes in the config file are only picked up after running `./bin/ipc_agent reload-config` so be sure to run it after editing your config.

Finally, to test if the connection to the rootnet has been successful, we can for instance try to create a new wallet in the rootnet: 
```
./bin/ipc_agent wallet-new --key-type=bls --subnet=/root
```

### Running a subnet
To spawn a new subnet, our IPC agent should be connected to at least the subnet of the parent we want to spawn the subnet from. You can refer to the previous section for information on how to run or connect to a rootnet. This instructions will assume the deployment of a subnet from `/root`, but the steps are equivalent for any other parent subnet. 

#### Spawn subnet actor
To run a subnet the first thing is to configure and create the subnet actor that will govern the subnet's operation:
> TODO: Update instructions when the new IPC actor bundle is deployed as some of these parameters have changed.
```bash
./bin/ipc_agent create-subnet -p <parent-id> -n <subnet-name> --min-validator-stake 1 --min-valid
ators 0 --finality-threshold 10 --check-period 10

# Sample command execution
./bin/ipc_agent create-subnet -p /root -n test --min-validator-stake 1 \
--min-validators 0 --finality-threshold 10 --check-period 10

[2023-03-21T09:32:58Z INFO  ipc_agent::cli::commands::manager::create] created subnet actor with id: /root/t01002
```
This command deploys a subnet actor for a new subnet from the `root`, with a human-readable name `test`, that requires at least `1` validator to join the subnet to be able to mine new blocks, and with a checkpointing period to the parent of `10` blocks. We can see that the output of this command is the ID of the new subnet.

#### Deploy subnet daemon
Before joining a new subnet, our node for that subnet should be initialized, because as part of the joining process we would need to provide information about our validator network address, so other validators know how to dial them. For the deployment of subnet daemons we also provide a convenient infra script: 
```bash
$ ./bin/ipc-infra/run-subnet-docker.sh <lotus-api-port> <validator-libp2p-port> <subnet-id> <absolute-path-validator-key>

# Sample execution
$ ./bin/ipc-infra/run-subnet-docker.sh 1239 1349 /root/t01002 /home/workspace/pl/lotus/scripts/ipc/src/wallet.key
```
> Beware: This script doesn't support the use of relative paths for the wallet path.

The end of the log of the execution of this script provides a bit more of information than the previous one as it is implemented to be used for production deployments:
```bash
>>> Subnet /root/t01002 daemon running in container: 22312347b743f1e95e50a31c1f47736580c9a84819f41cb4ed3d80161a0d750f
>>> Token to /root/t01002 daemon: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.TnoDqZJ1fqdkr_oCHFEXvdwU6kYR7Va_ALyEuoPnksA
>>> Default wallet: t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq
>>> Subnet subnet validator info:
t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq@/ip4/172.17.0.3/udp/1348/quic/p2p/12D3KooWN5hbWkCxwvrX9xYxMwFbWm2Jpa1o4qhwifmS t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq@/ip4/127.0.0.1/udp/1348/quic/p2p/12D3KooWN5hbWkCxwvrX9xYxMwFbWm2Jpa1o4qhwifmS t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq@/ip4/172.17.0.3/tcp/1347/p2p/12D3KooWN5hbWkCxwvrX9xYxMwFbWm2Jpa1o4qhwifmSw3Fb t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq@/ip4/127.0.0.1/tcp/1347/p2p/12D3KooWN5hbWkCxwvrX9xYxMwFbWm2Jpa1o4qhwifmSw3FbaVcL
>>> API listening in host port 1239
>>> Validator listening in host port 1349
```
> Beware: The validator address specified here should be the same as the one that will be used in the next step to join the subnet.

This log provides information about the API and auth tokens for the daemon, default validator wallet used, the multiaddresses where the validator is listening, etc. To configure our IPC agent with this subnet daemon, we need to once again update our IPC agent with the relevant information. In this case, for the sample execution above we need to add the following section to the end of our config file:
```toml
[subnets."/root/t01002"]
id = "/root/t01002"
jsonrpc_api_http = "http://127.0.0.1:1239/rpc/v1"
jsonrpc_api_ws = "wss://example.org/rpc/v0"
auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.TnoDqZJ1fqdkr_oCHFEXvdwU6kYR7Va_ALyEuoPnksA"
accounts = ["t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq"]
```
As always, remember to run `./bin/ipc_agent reload-config` for changes in the config of the agent to be picked up by the daemon.

#### Exporting wallet keys from subnet
In order to export the validator key from a wallet that may live in some other subnet into a file (like the wallet address we are using in the rootnet), we can use the following Lotus command:
```bash
eudico wallet export --lotus-json <address-to-export> > <output file>

# Sample execution
eudico wallet export --lotus-json t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq > /tmp/wallet.key
```
If your daemon is running on a docker container, you can get the container id (provided also in the output of the infra scripts), and run the following command above inside a container outputting the exported private key into a file locally:
```bash
$ docker exec -it <container-id> eudico wallet export --lotus-json <adress-to-export> > /tmp/wallet.key

# Sample execution
$ docker exec -it 84711d67cf162e30747c4525d69728c4dea8c6b4b35cd89f6d0947fee14bf908 eudico wallet export --lotus-json t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq > /tmp/wallet.key
```


#### Joining a subnet
With the daemon for the subnet deployed, we can join the subnet:
```bash
$ ./bin/ipc_agent join-subnet --subnet=<subnet-id> --collateral=<collateral_amount> --validator-net-addr=<libp2p-add-validator>

# Sample execution
$ ./bin/ipc_agent join-subnet --subnet=/root/t01002 --collateral=2 --validator-net-addr="/dns/host.docker.internal/tcp/1349/p2p/12D3KooWN5hbWkCxwvrX9xYxMwFbWm2Jpa1o4qhwifmSw3Fb"
```
This command specifies the subnet to join, the amount of collateral to provide and the validator net address used by other validators to dial them. We can pick up this information from the execution of the script above or running `eudico validator config validator-addr` from your deployment. Bear in mind that the multiaddress provided for the validator needs to be accessible publicly by other validators. According to the deployment used you may need to tweak the IP addresses of the multiaddresses and the ones provided by these scripts and commands won't be usable out-of-the-box.

For instance, in the example above, we are using the DNS endpoint `/dns/host.docker.internal/` so other Docker containers for the subnet deployed in the host machine know how to contact the validator.

As a sanity-check that we have joined the subnet successfully and that we provided enough collateral to register the subnet to IPC, we can list the child subnets of our parent with the following command:
```bash

$ ./bin/ipc_agent list-subnets --gateway-address=<gateway-addr> --subnet-id=<parent-subnet-id>

# Sample execution
$ ./bin/ipc_agent list-subnets --gateway-address=t064 --subnet-id=/root

[2023-03-22T15:42:22Z INFO  ipc_agent::cli::commands::manager::list_subnets] found child subnets: {"/root/t01002": SubnetInfoWrapper { id: "/root/t01002", stake: 2000000000000000000, circ_supply: 0, status:
```

> Note: In the current implementation of IPC the gateway actor is deployed as a system actor on the default addres `t064`, so whenever one of the IPC commands requests the address of the gateway actor you can use that value.

#### Mining in a subnet.
With our subnet daemon deployed, and having joined the network, as the minimum number of validators we set for our subnet is 0, we can start mining and creating new blocks in the subnet. Doing so is a simple as running the following script using as an argument the container of our subnet node: 
```bash
$  ./bin/ipc-infra/mine-subnet.sh <node-container-id>

# Sample execution
$  ./bin/ipc-infra/mine-subnet.sh 84711d67cf162e30747c4525d69728c4dea8c6b4b35cd89f6d0947fee14bf908
```

> TODO: The mining process is currently run in the foreground in interactive mode. Update infra scripts so they can be run detached and the logs are directed to a file.

#### Leaving a subnet
To leave a subnet, the following agent command can be used:
```bash
$ ./bin/ipc_agent leave-subnet --subnet=<subnet-id>

# Sample execution
$ ./bin/ipc_agent leave-subnet --subnet=/root/t01002
```

Leaving a subnet will release the collateral for the validator and remove all the validation rights from its account. This means that if you have a validator running in that subnet, its validation process will immediately terminate.

### Running a several nodes subnet

