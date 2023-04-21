#!/usr/bin/env bash
# Generated from topology/simple.yaml
set -e
# Create the agent(s)
make agent/up IPC_AGENT_NR=0
# Create the root node(s)
make node/up IPC_NODE_NR=0 IPC_SUBNET_NAME=head
# Alternate connecting agents and creating subnets and nodes to run them
make connect IPC_AGENT_NR=0 IPC_NODE_NR=0
make node/up IPC_AGENT_NR=0 IPC_NODE_NR=1 IPC_PARENT_NR=0 IPC_WALLET_NR=0 FUND_AMOUNT=5 IPC_SUBNET_NAME=thorax MIN_VALIDATOR_STAKE= MIN_VALIDATORS= BOTTOMUP_CHECK_PERIOD= TOPDOWN_CHECK_PERIOD=
make connect IPC_AGENT_NR=0 IPC_NODE_NR=1
make node/up IPC_AGENT_NR=0 IPC_NODE_NR=2 IPC_PARENT_NR=1 IPC_WALLET_NR=0 FUND_AMOUNT=3 IPC_SUBNET_NAME=abdomen MIN_VALIDATOR_STAKE= MIN_VALIDATORS= BOTTOMUP_CHECK_PERIOD= TOPDOWN_CHECK_PERIOD=
make connect IPC_AGENT_NR=0 IPC_NODE_NR=2
