#!/usr/bin/env bash
# Generated from topologies/simple.yaml
set -e
# Create the agent(s)
make --no-print-directory agent/up IPC_AGENT_NR=0
# Create the root node(s)
make --no-print-directory node/up IPC_NODE_NR=0 IPC_SUBNET_NAME=head
# Alternate connecting agents and creating subnets and nodes to run them
make --no-print-directory connect IPC_AGENT_NR=0 IPC_NODE_NR=0
make --no-print-directory node/up subnet/join subnet/fund IPC_AGENT_NR=0 IPC_NODE_NR=1 IPC_PARENT_NR=0 IPC_WALLET_NR=0 IPC_SUBNET_NAME=thorax WALLET_FUNDS=10 SUBNET_FUNDS=5 COLLATERAL=1 MIN_VALIDATOR_STAKE=1 MIN_VALIDATORS=0 BOTTOMUP_CHECK_PERIOD=10 TOPDOWN_CHECK_PERIOD=10
make --no-print-directory connect IPC_AGENT_NR=0 IPC_NODE_NR=1
