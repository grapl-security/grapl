#!/usr/bin/env bash

set -euo pipefail

target_container_name="${1}"

docker run \
    --interactive \
    --tty \
    --rm \
    --pid="container:${target_container_name}" \
    --net="container:${target_container_name}" \
    --privileged \
    --cap-add sys_admin \
    --cap-add sys_ptrace \
    --volume="$(pwd):/from-host" \
    --workdir="/from-host" \
    debugger:dev
