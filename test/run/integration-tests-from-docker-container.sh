#!/bin/bash

# This script is meant to be run by the CMD in
# python-integration-tests

set -euo pipefail

run_pants_python_tests() {
    local -r pytest_args="$1"
    ./pants filter --filter-target-type='python_tests' :: |
        xargs ./pants --tag='-needs_work' test --pytest-args="${pytest_args}"
}

# -rA means "report stdout/stderr for all tests, not just failing tests"
# -m integration_test limits it to integration tests.
# --no-cov disables coverage (since it's integration tests, makes sense!)
run_pants_python_tests "-m integration_test -rA --no-cov"
