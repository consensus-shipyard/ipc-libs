#!/usr/bin/env bash

# Send funds into a subnet.
# Call it on the child subnet node, so we can figure out from .env what the subnet ID is.

set -e

if [ $# -ne 4 ]
then
    echo "usage: ./fund-subnet.sh <agent-dir> <node-dir> <ipc-agent> <ipc-agent-url>"
    exit 1
fi

IPC_AGENT_DIR=$1
IPC_NODE_DIR=$2
IPC_AGENT=$3
IPC_AGENT_URL=$4

source $IPC_AGENT_DIR/.env
source $IPC_NODE_DIR/.env

# Rest of the variables from env vars.
IPC_SUBNET_FUNDS=${IPC_SUBNET_FUNDS:-0}

IPC_WALLET_DIR=$(dirname $IPC_WALLET_KEY)
ADDR=$(cat $IPC_WALLET_DIR/address)

run() {
  echo $@
  $@
}

if [ "$IPC_SUBNET_FUNDS" != "0" ]; then
  echo "[*] Funding $IPC_SUBNET_ID ($IPC_SUBNET_NAME) by wallet-$IPC_WALLET_NR ($ADDR) with $IPC_SUBNET_FUNDS token(s) using agent-$IPC_AGENT_NR"
  run $IPC_AGENT cross-msg fund --ipc-agent-url $IPC_AGENT_URL --subnet $IPC_SUBNET_ID --from $ADDR $IPC_SUBNET_FUNDS
else
  echo "[*] Fund amount is zero; skip sneding funds to $ADDR in $IPC_SUBNET_ID"
fi