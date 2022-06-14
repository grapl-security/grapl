#!/usr/bin/env bash

# This script sets up ssh via ssm to a server type (One of Consul Server, Nomad Agent or Nomad Server).
# This is a temporary workaround until we set up a VPN
#
# Input (optional): Port to forward to. This will be both the port on the server to forward and the local port that is forwarded to. Defaults to port 1234

set -euo pipefail

readonly SERVER_TO_SSM_TO=$1

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/lib/ssm_tools.sh"

ssm_ssh "${SERVER_TO_SSM_TO}"
