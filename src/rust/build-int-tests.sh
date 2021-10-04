#!/bin/bash

# Build Rust integration tests and copy their binaries to dist/tests.
# The dist/tests directory will be used for packaging these tests
# in a container image.

set -euo pipefail

THIS_DIR="$(dirname "${BASH_SOURCE[0]}")"
OUT_DIR="${THIS_DIR}/dist/tests"

TEST_PATHS=$(cargo test \
    --features "node-identifier/integration,sqs-executor/integration,kafka-metrics-exporter/integration" \
    --no-run \
    --message-format=json \
    --test "*" | \
    jq -r "select(.profile.test == true) | .filenames[]")

# Create output directory if it doesn't exist
mkdir --parents "${OUT_DIR}"
# Clear the output directory if it previously existed
rm -f "${OUT_DIR}/{.*,*}"

for path in ${TEST_PATHS}; do
    echo Copying "${path}" to ${OUT_DIR}
    cp "${path}" "${OUT_DIR}/"
done

