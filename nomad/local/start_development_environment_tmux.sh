#!/bin/bash

# This file is an alternative version of `start_development_environment.sh` that uses
# tmux to split up logs from Nomad-Agent, Consul, and the nomad deployment itself.

set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
cd "${THIS_DIR}"

# Ensure script is being run with `local-grapl.env` variables
# via `make start-nomad-dev`
if [[ ! -v DOCKER_REGISTRY ]]; then
    echo "!!! Run this with 'make start-nomad-dev'"
    exit 1
fi

# ensure that all dependencies are available
if [[ -z $(command -v nomad) ]]; then
    echo "Nomad must be installed. Please follow the install instructions at https://www.nomadproject.io/downloads"
    exit 2
fi

if [[ -z $(command -v consul) ]]; then
    echo "Consul must be installed. Please follow the install instructions at https://www.consul.io/downloads"
    exit 2
fi

if [[ -z $(command -v tmuxinator) ]]; then
    echo "tmuxinator must be installed. sudo apt-get install tmuxinator"
    exit 2
fi

# NOTE: tmux 2.8 in apt works fine, but 3 enables features like named panes.
# Worth exploring/enforcing.
tmuxinator start project nomad-development-environment
