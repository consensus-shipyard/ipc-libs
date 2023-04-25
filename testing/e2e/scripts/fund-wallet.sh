#!/usr/bin/env bash

# Fund an existing wallet with funds from the default wallet of the agent.
# Call it on the node where the wallet will receive the funds.

set -e

if [ $# -ne 5 ]
then
    echo "usage: ./fund-wallet.sh <agent-dir> <node-dir> <wallet-dir> <ipc-agent> <ipc-agent-url>"
    exit 1
fi

IPC_AGENT_DIR=$1
IPC_NODE_DIR=$2
IPC_WALLET_DIR=$3
IPC_AGENT=$4
IPC_AGENT_URL=$5

source $IPC_NODE_DIR/.env
source $IPC_AGENT_DIR/.env

# Rest of the variables from env vars.
IPC_WALLET_FUNDS=${IPC_WALLET_FUNDS:-0}

ADDR=$(cat $IPC_WALLET_DIR/address)

run() {
  echo $@
  $@
}

if [ "$IPC_WALLET_FUNDS" != "0" ]; then
  echo "[*] Funding wallet-$IPC_WALLET_NR ($ADDR) with $IPC_WALLET_FUNDS token(s) using agent-$IPC_AGENT_NR on $IPC_NODE_TYPE node-$IPC_NODE_NR under $IPC_SUBNET_ID ($IPC_SUBNET_NAME)"
  run $IPC_AGENT subnet send-value --ipc-agent-url $IPC_AGENT_URL --subnet $IPC_SUBNET_ID --to $ADDR $IPC_WALLET_FUNDS
else
  echo "[*] Fund amount is zero; skip funding $ADDR"
fi
