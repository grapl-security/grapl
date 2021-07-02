#!/usr/bin/env bash

oneTimeSetUp() {
    # shellcheck source-path=SCRIPTDIR
    source "$(dirname "${BASH_SOURCE[0]}")/util.sh"
}

test_is_real_run() {
    unset BREAK_GLASS_IN_CASE_OF_EMERGENCY
    local BUILDKITE_SOURCE

    BUILDKITE_SOURCE=webhook
    assertTrue "webhook-initiated runs are considered \"real\"" is_real_run

    BUILDKITE_SOURCE=api
    assertTrue "api-initiated runs are considered \"real\"" is_real_run

    BUILDKITE_SOURCE=trigger_job
    assertTrue "trigger-initiated runs are considered \"real\"" is_real_run

    BUILDKITE_SOURCE=schedule
    assertTrue "scheduled runs are considered \"real\"" is_real_run

    BUILDKITE_SOURCE=ui
    assertFalse "UI-initiated runs are *not* considered \"real\"" is_real_run

    BUILDKITE_SOURCE=not_a_valid_buildkite_source
    assertFalse "A run with an unrecognized BUILDKITE_SOURCE is never considered \"real\"" is_real_run
}

test_is_real_run_with_override() {
    local BREAK_GLASS_IN_CASE_OF_EMERGENCY=1
    local BUILDKITE_SOURCE

    BUILDKITE_SOURCE=webhook
    assertTrue "webhook-initiated runs are considered \"real\", regardless of override" is_real_run

    BUILDKITE_SOURCE=api
    assertTrue "api-initiated runs are considered \"real\", regardless of override" is_real_run

    BUILDKITE_SOURCE=trigger_job
    assertTrue "trigger-initiated runs are considered \"real\", regardless of override" is_real_run

    BUILDKITE_SOURCE=schedule
    assertTrue "scheduled runs are considered \"real\", regardless of override" is_real_run

    BUILDKITE_SOURCE=ui
    assertTrue "UI-initiated runs are *only* considered \"real\" in the presence of an override" is_real_run

    BUILDKITE_SOURCE=not_a_valid_buildkite_source
    assertFalse "A run with an unrecognized BUILDKITE_SOURCE is never considered \"real\"" is_real_run
}
