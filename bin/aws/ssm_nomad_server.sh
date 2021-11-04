#!/usr/bin/env bash

# This script sets up port forwarding between an arbitrary nomad server and your local computer.
# This is a temporary workaround until we set up a VPN
# This will allow you to view the nomad UI and interact with nomad in the cli. In particular, this is used by pulumi to deploy Nomad jobs
#
# Input (optional): Local port to forward to. Defaults to port 4646

set -euo pipefail

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/lib/ssm_tools.sh"

LOCAL_PORT_TO_FORWARD_TO="${1:-4646}"
REMOTE_PORT=4646

ssm_port_forward "Nomad Server" "${LOCAL_PORT_TO_FORWARD_TO}" "${REMOTE_PORT}"
