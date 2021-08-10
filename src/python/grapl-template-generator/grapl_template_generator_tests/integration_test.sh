#!/usr/bin/env bash

# This script is meant to be run in a real, honest-to-goodness shell.
# No Docker, no Pants, just a real Buildkite host or dev machine.

set -euo pipefail

# Make sure our tree is clean.
git diff HEAD --exit-code > /dev/null 2>&1 || (echo "Your git tree is dirty; bailing!" && exit 42)

GRAPL_ROOT="$(git rev-parse --show-toplevel)"
cd "${GRAPL_ROOT}"

make grapl-template-generator
./bin/grapl-template-generator --hax-update-cargo-toml cool-test-service

# Did the new service show up in cargo.toml?
grep cool-test-service ./src/rust/Cargo.toml

# Cool, that means we can `make build-test-rust` and ensure it all compiled correctly.
make build-test-unit-rust

# Do some clean-up on the user's behalf
rm -r ./src/proto/graplinc/grapl/api/cool_test_service/
rm -r ./src/rust/cool-test-service
git checkout HEAD -- src/rust/Cargo.toml