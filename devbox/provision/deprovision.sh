#!/usr/bin/env bash

set -euo pipefail

readonly GRAPL_ROOT="${PWD}"
THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/../lib.sh"

pulumi destroy --yes --cwd="${GRAPL_ROOT}/devbox/provision"

rm -rf "${GRAPL_DEVBOX_DIR}"