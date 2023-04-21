#!/usr/bin/env bash

set -e

if [ $# -ne 1 ]
then
    echo "usage: ./topology.sh <topology-yaml-path>"
    exit 1
fi

TOPO_YAML=$1
TOPO_JSON=$(dirname $TOPO_YAML)/$(basename $TOPO_YAML .yaml).json
TOPO_SH=$(dirname $TOPO_JSON)/$(basename $TOPO_JSON .json).sh

echo "[*] Compiling $TOPO_YAML to $TOPO_SH"

yq -Poj $TOPO_YAML > $TOPO_JSON

echo "#!/usr/bin/env bash" > $TOPO_SH
echo "# Generated from $TOPO_YAML" >> $TOPO_SH
echo "set -e" >> $TOPO_SH

echo "# Create the agent(s)" >> $TOPO_SH
cat $TOPO_JSON | jq -r '
  .agents[]
  | "make agent/up IPC_AGENT_NR=" + (.nr | tostring)
' >> $TOPO_SH

echo "# Create the root node(s)" >> $TOPO_SH
cat $TOPO_JSON | jq -r '
  .nodes[]
  | select((.parent_node == .nr) or (. | has("parent_node") | not))
  | "make node/up IPC_NODE_NR=" + (.nr | tostring) + " IPC_SUBNET_NAME=" + (.subnet.name | tostring)
' >> $TOPO_SH

echo "# Alternate connecting agents and creating subnets and nodes to run them" >> $TOPO_SH
cat $TOPO_JSON | jq -r '
  . as $top
  |
      [
        $top.agents[]
        | . as $agent
        | .connections[]
        | {
            sort_key: ((.node | tostring) + "/a"),
            node: .node,
            agent: $agent.nr,
            cmd: ("make connect IPC_AGENT_NR=" + ($agent.nr | tostring)
                              + " IPC_NODE_NR="  + (.node | tostring))
          }
      ] as $connections
    | $connections
    | map(. | { key: .node|tostring, value: .agent|tostring })
    | from_entries as $node_agent_map
    | [
        $top.nodes[]
        | select(has("parent_node") and (.parent_node != .nr))
        | {
            sort_key: ((.parent_node | tostring) + "/b"),
            cmd: ("make node/up IPC_AGENT_NR="    + ($node_agent_map[.parent_node | tostring])
                            + " IPC_NODE_NR="     + (.nr | tostring)
                            + " IPC_PARENT_NR="   + (.parent_node | tostring)
                            + " IPC_WALLET_NR="   + (.wallet | tostring)
                            + " FUND_AMOUNT="     + (.fund_amount | tostring)
                            + " IPC_SUBNET_NAME=" + (.subnet.name)
                            + " MIN_VALIDATOR_STAKE="   + (.subnet | if has("min_validator_stake")   then .min_validator_stake   | tostring else "" end)
                            + " MIN_VALIDATORS="        + (.subnet | if has("min_validators")        then .min_validators        | tostring else "" end)
                            + " BOTTOMUP_CHECK_PERIOD=" + (.subnet | if has("bottomup_check_period") then .bottomup_check_period | tostring else "" end)
                            + " TOPDOWN_CHECK_PERIOD="  + (.subnet | if has("topdown_check_period")  then .topdown_check_period  | tostring else "" end) )
          }
      ] as $subnets
    | [
        $connections | map({sort_key: .sort_key, cmd: .cmd}),
        $subnets
      ]
    | flatten(1)
    | sort_by(.sort_key)
    | .[]
    | .cmd
' >> $TOPO_SH

rm $TOPO_JSON
