#!/usr/bin/env bash

# Create a new subnet on a node and export its ID.
# Call it on the node running the parent subnet.

set -e

if [ $# -ne 5 ]
then
    echo "usage: ./new-subnet.sh <agent-dir> <node-dir> <subnet-dir> <ipc-agent> <ipc-agent-url>"
    exit 1
fi

IPC_AGENT_DIR=$1
IPC_NODE_DIR=$2
IPC_SUBNET_DIR=$3
IPC_AGENT=$4
IPC_AGENT_URL=$5

source $IPC_AGENT_DIR/.env
source $IPC_NODE_DIR/.env

IPC_SUBNET_NAME=$(basename $IPC_SUBNET_DIR)

# Rest of the variables from env vars.
MIN_VALIDATOR_STAKE=${MIN_VALIDATOR_STAKE:-1}
MIN_VALIDATORS=${MIN_VALIDATORS:-0}
BOTTOMUP_CHECK_PERIOD=${BOTTOMUP_CHECK_PERIOD:-10}
TOPDOWN_CHECK_PERIOD=${TOPDOWN_CHECK_PERIOD:-10}

echo "[*] Creating new subnet with agent-$IPC_AGENT_NR on $IPC_NODE_TYPE node-$IPC_NODE_NR under $IPC_SUBNET_ID named $IPC_SUBNET_NAME"

CMD=$(echo $IPC_AGENT subnet create --ipc-agent-url $IPC_AGENT_URL --parent $IPC_SUBNET_ID --name $IPC_SUBNET_NAME --min-validator-stake $MIN_VALIDATOR_STAKE --min-validators $MIN_VALIDATORS --bottomup-check-period $BOTTOMUP_CHECK_PERIOD --topdown-check-period $TOPDOWN_CHECK_PERIOD)
echo $CMD
set +e
LOG=$($CMD 2>&1)
STATUS=$?
if [ $STATUS != 0 ]; then
    echo $LOG
    exit 1
fi
set -e

# Example output from the agent:
# [2023-04-17T11:44:13Z INFO  ipc_agent::cli::commands::subnet::create] created subnet actor with id: /root/t01003
IPC_SUBNET_ID=$(echo $LOG | sed 's/^.*id: \(\/root\/.*\)$/\1/')

if [ -z "$IPC_SUBNET_ID" ]; then
    echo "ERROR: Could not find the subnet ID in the logs.";
    exit 1
fi

echo "[*] Writing details for $IPC_SUBNET_NAME to $IPC_SUBNET_DIR"
mkdir -p $IPC_SUBNET_DIR
echo $IPC_SUBNET_ID > $IPC_SUBNET_DIR/id
