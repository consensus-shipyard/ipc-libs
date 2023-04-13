#!/usr/bin/env bash

set -e

# By this time the daemon should have written to the LOTUS_PATH
# the API token we can use to contact the server.
while [ ! -f $LOTUS_PATH/token ]; do
  echo "Waiting for the API token to appear...";
  sleep 5
done

API_TOKEN=$(cat $LOTUS_PATH/token)

# Set the env var that Lotus is looking for.
export FULLNODE_API_INFO=${API_TOKEN}:/dns/${DAEMON_HOSTNAME}/tcp/1234/http

if [ "${IPC_SUBNET_ID}" == "/root" ]; then
  echo "Running as root net..."
  exec /scripts/ipc/src/root-single-validator.sh
else
  echo "Running as subnet..."
  exec /scripts/ipc/src/subnet-validator.sh
fi
