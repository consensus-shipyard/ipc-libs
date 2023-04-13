#!/usr/bin/env bash

set -e

if [ "${IPC_SUBNET_ID}" == "/root"]; then
  exec /scripts/ipc/src/root-single-validator.sh
else
  exec /scripts/ipc/src/subnet-validator.sh
fi
