#!/usr/bin/env bash

# Determines whether this pipeline run is "real" or if it was an
# ad-hoc one triggered by a person for testing. If the latter, there
# are some actions that we don't want to perform (such as pushing a
# new merge commit to an `rc` branch, performing a release, etc.).
#
# In the rare case that a human *does* need to legitimately run a
# pipeline *as though it were a real run*, then the
# `BREAK_GLASS_IN_CASE_OF_EMERGENCY` environment variable should be
# set.
#
# Hinges on the value of `BUILDKITE_SOURCE`; see
# https://buildkite.com/docs/pipelines/environment-variables#bk-env-vars-buildkite-source
# for details.
is_real_run() {
    case "${BUILDKITE_SOURCE}" in
        webhook | api | trigger_job | schedule)
            true
            ;;
        ui)
            if [ -n "${BREAK_GLASS_IN_CASE_OF_EMERGENCY:-}" ]; then
                true
            else
                false
            fi
            ;;
        *)
            echo "--- :exclamation_mark: Unrecognized BUILDKITE_SOURCE: ${BUILDKITE_SOURCE}; cowardly refusing to consider this run \"real\""
            false
            ;;
    esac
}
