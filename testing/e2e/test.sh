#!/usr/bin/env bash

set -e

run() {
  echo $@
  $@
}

run exit 1
