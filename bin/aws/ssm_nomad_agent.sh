#!/usr/bin/env bash

# This script sets up port forwarding between an arbitrary nomad agent server and your local computer.
# This is a temporary workaround until we set up a VPN
# This will allow you to view the UI of a container on the server and/or interact with the container via the cli
#
# Input (optional): Port to forward to. This will be both the port on the server to forward and the local port that is forwarded to. Defaults to port 1234

set -euo pipefail

# shellcheck source=bin/aws/lib/ssm_tools.sh
source lib/ssm_tools.sh

PORT_TO_FORWARD="${1:-1234}"

ssm_port_forward "Nomad Agent" "${PORT_TO_FORWARD}" "${PORT_TO_FORWARD}"
