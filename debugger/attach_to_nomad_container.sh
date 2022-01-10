#!/usr/bin/env bash

set -euo pipefail

########################################################################
# See `grapl-local-infra.nomad` `debugger` section to understand usecase.
########################################################################

DEBUGGER_CONTAINER=$(docker ps --filter ancestor=debugger:dev --format="{{.Names}}")

docker exec --interactive --tty "${DEBUGGER_CONTAINER}" /bin/bash
