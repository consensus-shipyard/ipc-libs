#!/usr/bin/env bash

set -e

if [ "${IPC_SUBNET_ID}" == "/root" ]; then
  echo "running as root"
  exec /scripts/ipc/entrypoints/eudico-root-single.sh
else
  echo "running as subnet"
  exec /scripts/ipc/entrypoints/eudico-subnet.sh $IPC_SUBNET_ID
fi
