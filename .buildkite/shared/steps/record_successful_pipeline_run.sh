#!/usr/bin/env bash

# At the end of a successful run of the pipeline, we'll want to
# record the git SHA of the code we were running.
#
# This is to ensure that we can make the appropriate decision whether
# or not to rebuild artifacts following a *failure* of this pipeline
# for whatever reason; see the `diff.sh` script for additional
# details.

set -euo pipefail

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/../lib/record.sh"

tag_last_success "$(pipeline_from_env)"
