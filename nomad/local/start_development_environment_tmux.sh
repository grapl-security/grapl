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

# This guard is strictly informative. nomad agent -dev-connect cannot run without root
if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root"
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

if [[ -z $(command -v tmux) ]]; then
    echo "tmux must be installed. sudo apt-get install tmux!"
    exit 2
fi

# -d = detached; -s = session name; -t = target window
SESSION="start_development_environment"
WINDOW="${SESSION}:0"
tmux kill-session -t "${SESSION}" || true
tmux new-session -d -s "${SESSION}"
tmux set -g mouse on

# Setup your panes
tmux split-window -h -p 20
tmux split-window -v
LEFT="${WINDOW}.0"
TOP_RIGHT="${WINDOW}.1"
BOTTOM_RIGHT="${WINDOW}.2"

# Kick off Nomad Agent
tmux send-keys -t $TOP_RIGHT \
    'nomad agent -config="nomad-agent-conf.nomad" -dev-connect' \
    Enter

# Kick off Consul agent
tmux send-keys -t $BOTTOM_RIGHT \
    'consul agent -dev' \
    Enter

# Deploy your infra
tmux send-keys -t $LEFT \
    "echo '--- To kill session: Ctrl-B + :kill-session'" Enter \
    "echo '--- To change windows: Ctrl-B + w'" Enter \
    "./nomad_run_local_infra.sh" Enter

tmux switch-client -t "${SESSION}" ||
    tmux attach-session -t "${SESSION}"

# After we `kill-session` the tmux, we fall back to here
killall nomad || true
echo "hey"
killall consul || true
echo "hi"
