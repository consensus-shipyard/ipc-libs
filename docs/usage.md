# Using the IPC Agent

>💡 For background and setup information, make sure to start with the [README](/README.md).

## Listing active subnets

As a sanity-check that we have joined the subnet successfully and that we provided enough collateral to register the subnet to IPC, we can list the child subnets of our parent with the following command:

```bash
$ ./bin/ipc-agent list-subnets --gateway-address=<gateway-addr> --subnet=<parent-subnet-id>

# Sample execution
$ ./bin/ipc-agent list-subnets --gateway-address=t064 --subnet=/root
[2023-03-30T17:00:25Z INFO  ipc_agent::cli::commands::manager::list_subnets] /root/t01003 - status: 0, collateral: 2 FIL, circ.supply: 0.0 FIL
```

This command only shows subnets that have been registered to the gateway, i.e. that have provided enough collateral to participate in the IPC protocol and haven't been killed. It is not an exhaustive list of all of the subnet actors deployed over the network.

## Joining a subnet

With the daemon for a subnet deployed (see [instructions](/docs/subnet.md)), one can join the subnet:
```bash
$ ./bin/ipc-agent subnet join --subnet <subnet-id> --collateral <collateral_amount> --validator-net-addr <libp2p-add-validator>

# Example execution
$ ./bin/ipc-agent subnet join --subnet /root/t01002 --collateral 2 --validator-net-addr /dns/host.docker.internal/tcp/1349/p2p/12D3KooWN5hbWkCxwvrX9xYxMwFbWm2Jpa1o4qhwifmSw3Fb
```
This command specifies the subnet to join, the amount of collateral to provide and the validator net address used by other validators to dial them.

## Sending funds in a subnet

The agent provides a command to conveniently exchange funds between addresses of the same subnet. This can be achieved through the following command:
```bash
$ ./bin/ipc-agent subnet send-value --subnet <subnet-id> --to <to-addr> <value>

# Example execution
$ ./bin/ipc-agent subnet send-value --subnet /root/t01002 --to t1xbevqterae2tanmh2kaqksnoacflrv6w2dflq4i 10
```

## Sending funds between subnets

At the moment, the IPC agent only expose commands to perform the basic IPC interoperability primitives for cross-net communication, which is the exchange of FIL (the native token for IPC) between the same address of a subnet. Mainly:
- `fund`, which sends FIL from one public key address, to the same public key address in the child.
- `release` that moves FIL from one account in a child subnet to its counter-part in the parent.
Through this basic primitives, any complex behavior can be implemented. Sending value to some other account in another subnet can be implemented a set of `fund/release` and `sendValue` operations. Calling  smart contract from one subnet to another works by providing funds to one account in the destination subnet, and then calling the contract. The agent doesn't currently include abstractions for this complex operations (although it will in the future). That being said, users can still leverage the agent's API to easily compose the basic primitives into complex functionality (the docs will be extended soon).

> Note: All cross-net operations need to pay an additional cross-msg fee (apart from the gas cost of the message). This is reason why even if you sent `X FIL` you may see `X - fee FIL` arriving to you account at destination. This fee is used to reward subnet validator for their work committing the checkpoints that forwarded the message to its destination.

### Fund
Funding a subnet can be performed by using the following command:
```bash
$ ./bin/ipc-agent cross-msg fund --subnet=<subnet-id> <amount>

# Example execution
$ ./bin/ipc-agent cross-msg fund --subnet=/root/t01002 100

```
This command includes the cross-net message into the next top-down checkpoint after the current epoch. Once the top-down checkpoint is conveniently committed, you should see the funds in your account of the child subnet.

> Top-down checkpoints are not used to anchor the security of the parent into the child (as it is the case for bottom-up checkpoints), they just include information of the top-down messages that need to beexecuted in the child subnet, and is just a way for validators in the subnet to reach consensus about the finality on their parent (that they all trust).

### Release
In order to release funds from a subnet, it should already have some circulating supply in it, and your account should have enough funds to perform the operation. This means, that unless there was already other activity in the subnet, a `fund` operation needs to be trigger to send some funds to the subnet. The circulating supply of a subnet can be checked with the `list-subnets` command.
```bash
$ ./bin/ipc-agent cross-msg release --subnet=<subnet-id> <amount>

# Example execution
$ ./bin/ipc-agent cross-msg release --subnet=/root/t01002 100

```
This command includes the cross-net message into a bottom-up checkpoint after the current epoch. Once the bottom-up checkpoint is conveniently committed, you should see the funds in your account of the parent. You can check the checkpoint where the cross-message was included by listing the checkpoints around the epoch where your message was sent, as follows: 
```bash
# Example of checkpoint list command searching for the checkpoint including
# our cross-msg
$ ./bin/ipc-agent checkpoint list --from-epoch=10000 --to-epoch=16000 --subnet=/root/t01002
[2023-04-18T17:37:49Z INFO  ipc_agent::cli::commands::checkpoint::list_checkpoints] epoch 10500 - prev_check="bafy2bzaceb5nt5g7bnkjtrkchw3ehoj5meofa5qhs7rmpat7mu3bn5ymmfrha", cross_msgs={"cross_msgs":[],"fee":[]}, child_checks=[]
[2023-04-18T17:37:49Z INFO  ipc_agent::cli::commands::checkpoint::list_checkpoints] epoch 10510 - prev_check="bafy2bzacecflzfgombxgdaamnfznmckuhjsidnaaj6d2h6zbb575nn5mijrmy", cross_msgs={"cross_msgs":[],"fee":[]}, child_checks=[]
[2023-04-18T17:37:49Z INFO  ipc_agent::cli::commands::checkpoint::list_checkpoints] epoch 10520 - prev_check="bafy2bzacebdngpnuogln7uijrdtyz6nxxlzbcbsd7zbvkid2vntenjwdlvsak", cross_msgs={"cross_msgs":[{"from":"/root/t01002:t099","method":0,"nonce":2,"params":"","to":"/root:t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq","value":0}],"fee":[0,23,72,118,232,0]}, child_checks=[]
```

> TODO: Add a note on how `wallet list` can be used to check your funds in a subnet. This should be implemented tomorrow (if things go well).

## Listing checkpoints from a subnet

Subnets are periodically committing checkpoints to their parent every `check-period` (parameter defined when creating the subnet). If you want to inspect the information of a range of checkpoints committed in the parent for a subnet, you can use the `checkpoint list` command provided by the agent as follows: 
```bash
# List checkpoints between two epochs for a subnet
$ ./bin/ipc-agent checkpoint list --from-epoch <range-start> --to-epoch <range-end> --subnet <subnet-id>

# Example execution
$ ./bin/ipc-agent checkpoint list --from-epoch 0 --to-epoch 100 --subnet root/t01002
[2023-03-29T12:43:42Z INFO  ipc_agent::cli::commands::manager::list_checkpoints] epoch 0 - prev_check={"/":"bafy2bzacedkoa623kvi5gfis2yks7xxjl73vg7xwbojz4tpq63dd5jpfz757i"}, cross_msgs=null, child_checks=null
[2023-03-29T12:43:42Z INFO  ipc_agent::cli::commands::manager::list_checkpoints] epoch 10 - prev_check={"/":"bafy2bzacecsatvda6lodrorh7y7foxjt3a2dexxx5jiyvtl7gimrrvywb7l5m"}, cross_msgs=null, child_checks=null
[2023-03-29T12:43:42Z INFO  ipc_agent::cli::commands::manager::list_checkpoints] epoch 30 - prev_check={"/":"bafy2bzaceauzdx22hna4e4cqf55jqmd64a4fx72sxprzj72qhrwuxhdl7zexu"}, cross_msgs=null, child_checks=null
```

## Checking the health of top-down checkpoints
In order to check the health of top-down checkpointing in a subnet, the following command can be run:
```bash
$./bin/ipc-agent checkpoint last-td-exec --subnet=<subnet-id>

# Example execution
$./bin/ipc-agent checkpoint last-td-exec --subnet /root/t01002
[2023-04-18T17:11:34Z INFO  ipc_agent::cli::commands::checkpoint::topdown_executed] Last top-down checkpoint executed in epoch: 9866
```

This command return the epoch of the last top-down checkpoint executed in the child. If you see that this epoch is way below the current epoch of the parent of the child subnet, it means that the top-down checkpoint may be lagging, that validators need to catch-up, and that the forwarding of top-down messages (from parent to child) may take longer to be committed.


## Leaving a subnet

To leave a subnet, the following agent command can be used:
```bash
$ ./bin/ipc-agent subnet leave --subnet <subnet-id>

# Example execution
$ ./bin/ipc-agent subnet leave --subnet /root/t01002
```
Leaving a subnet will release the collateral for the validator and remove all the validation rights from its account. This means that if you have a validator running in that subnet, its validation process will immediately terminate.
