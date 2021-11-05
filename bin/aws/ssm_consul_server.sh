#!/usr/bin/env bash

# This script sets up port forwarding between an arbitrary consul server and your local computer.
# This is a temporary workaround until we set up a VPN
# This will allow you to view the consul UI and interact with consul in the cli
#
# Input (optional): Local port to forward to. Defaults to port 8500

set -euo pipefail

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/lib/ssm_tools.sh"

LOCAL_PORT_TO_FORWARD_TO="${1:-8500}"
REMOTE_PORT=8500

ssm_port_forward "Consul Server" "${LOCAL_PORT_TO_FORWARD_TO}" "${REMOTE_PORT}"
