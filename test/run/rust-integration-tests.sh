#!/bin/bash

# This script is intended to be used for running Rust integration tests.
# It is invoked from rust/Dockerfile.
# The exit code of this script will be that of
# that last test that had a non-zero exit code (test failure), otherwise it
# will be zero.

EXIT_STATUS=0
declare -a FAILING_TESTS=()

# shellcheck disable=SC2044
for test in $(find /tests -type f -executable -exec readlink -f {} \;); do
    echo "--- Executing ${test}"
    # Redirect stderr so it's inline with stdout
    "${test}" --test-threads=1
    exit_code=$?
    if [[ ${exit_code} -ne 0 ]]; then
        FAILING_TESTS+=("${test}")
        echo Test failed with exit code ${exit_code}
        EXIT_STATUS=${exit_code}
    fi
done

if [[ ${EXIT_STATUS} -ne 0 ]]; then
    echo ""
    echo ">>> FAILING TESTS: ${FAILING_TESTS[*]}"
    echo "   See .stderr.txt (or the Nomad stderr tab) to see additional failure backtraces."
fi
exit ${EXIT_STATUS}
