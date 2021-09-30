#!/usr/bin/env bash

# A diffing script for use with the chronotc/monorepo-diff Buildkite
# plugin, and aware of Grapl release pipeline conventions for verify
# and merge pipelines.
#
# In particular, this will modify the diff command appropriately for
# whether it is running in the context of a verify pipeline, or from
# the steps of a verify pipeline being run within a merge pipeline.
#
# The script will output the names of files changed. DO NOT echo
# anything else to standard output (e.g., logging statements), or it
# will be considered as a file that changed.

set -euo pipefail

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/../lib/record.sh"

pipeline="$(pipeline_from_env)"
readonly pipeline

case "${pipeline}" in
    merge)
        # If the last run of the merge pipeline failed, we still want
        # to perform the expensive operations we're using this script
        # to be selective about.
        #
        # For instance, the failing run may have been building Packer
        # AMI images, but failed due to an unrelated issue in the
        # build scripts, or something totally unrelated to the Packer
        # source files. The fix would then not touch those source
        # files, and we would never rebuild the image.
        git diff --name-only "$(tag_for_pipeline "${pipeline}")"
        ;;
    verify)
        # We're on a PR branch, so what changed on this branch
        # relative to main?
        #
        # > For example, origin.. is a shorthand for origin..HEAD and asks
        #   "What did I do since I forked from the origin branch?"
        #
        #   - from `man 7 gitrevisions`, "SPECIFYING RANGES"
        git diff --name-only main..
        ;;
    *)
        # We don't have any other pipelines at the moment, but we'd
        # like to fail once we do, to ensure we fix this script as
        # appropriate.
        exit 42
        ;;
esac
