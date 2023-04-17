# Topology compiled from topology/example.yaml
# Create the agent(s)
make agent/up IPC_AGENT_NR=0
make agent/up IPC_AGENT_NR=1
# Create the root node(s)
make node/up IPC_NODE_NR=0 IPC_SUBNET_NAME=vertebrates
# Alternate connecting agents and creating subnets and nodes to run them
make connect IPC_AGENT_NR=0 IPC_NODE_NR=0
make subnet IPC_AGENT_NR=0 IPC_NODE_NR=1 IPC_PARENT_NR=0 IPC_WALLET_NR=0 IPC_SUBNET_NAME=warm-blooded
make subnet IPC_AGENT_NR=0 IPC_NODE_NR=2 IPC_PARENT_NR=0 IPC_WALLET_NR=1 IPC_SUBNET_NAME=cold-blooded
make subnet IPC_AGENT_NR=0 IPC_NODE_NR=5 IPC_PARENT_NR=0 IPC_WALLET_NR=1 IPC_SUBNET_NAME=warm-blooded
make connect IPC_AGENT_NR=1 IPC_NODE_NR=1
make subnet IPC_AGENT_NR=1 IPC_NODE_NR=3 IPC_PARENT_NR=1 IPC_WALLET_NR=0 IPC_SUBNET_NAME=mammals
make subnet IPC_AGENT_NR=1 IPC_NODE_NR=4 IPC_PARENT_NR=1 IPC_WALLET_NR=1 IPC_SUBNET_NAME=birds
make connect IPC_AGENT_NR=0 IPC_NODE_NR=5
