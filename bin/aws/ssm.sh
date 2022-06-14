#!/usr/bin/env bash

# This script sets up ssh via ssm to a server type (One of Consul Server, Nomad Agent or Nomad Server).
# This is a temporary workaround until we set up a VPN
#
# Input: Server type. This should be one of "Consul Server", "Nomad Agent" or "Nomad Server"
# Usage: ./bin/aws/ssm.sh "Nomad Server"

set -euo pipefail

readonly SERVER_TO_SSM_TO=$1

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/lib/ssm_tools.sh"

ssm "${SERVER_TO_SSM_TO}"
