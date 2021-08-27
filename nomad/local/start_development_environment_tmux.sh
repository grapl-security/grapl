#!/bin/bash

# This file is an alternative version of `start_development_environment.sh` that uses
# tmux to split up logs from Nomad-Agent, Consul, and the nomad deployment itself.

set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
GRAPL_ROOT="$(git rev-parse --show-toplevel)"
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
tmux kill-session -t '=start_development_environment' || true
tmux new-session -d -s start_development_environment

tmux new-window -d -t '=start_development_environment' -n nomad_agent
tmux send-keys -t '=start_development_environment:=nomad_agent' \
    'nomad agent -config="nomad-agent-conf.nomad" -dev-connect' \
    Enter

tmux new-window -d -t '=start_development_environment' -n consul_agent
tmux send-keys -t '=start_development_environment:=consul_agent' \
    'consul agent -dev' \
    Enter

tmux new-window -d -t '=start_development_environment' -n nomad_client
tmux send-keys -t '=start_development_environment:=nomad_client' \
    "echo '--- To kill session: Ctrl-B + :kill-session'" Enter \
    "echo '--- To change windows: Ctrl-B + W'" Enter

tmux send-keys -t '=start_development_environment:=nomad_client' \
    "./nomad_run_local_infra.sh" \
    Enter

# Get rid of default window, it's useless
tmux kill-window -t '=start_development_environment:=0'
tmux switch-client -t '=start_development_environment' ||
tmux attach-session -t '=start_development_environment'

# After we `kill-session` the tmux, we fall back to here
killall nomad
killall consul