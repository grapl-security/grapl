#!/usr/bin/env bash

set -euo pipefail

source .buildkite/scripts/lib/artifacts.sh
source .buildkite/scripts/lib/cloudsmith.sh
source .buildkite/scripts/lib/version.sh

# This is the name defined in the top-level Makefile in the
# `ux-tarball` target.
readonly UX_FILENAME="grapl-ux.tar.gz"
readonly UX_ARTIFACTS_FILE="ux_artifacts.json"

echo "--- :yarn: Building Grapl UX Artifact"
make ux-tarball

version=$(timestamp_and_sha_version)
readonly version

echo "--- :cloudsmith: Uploading ${UX_FILENAME} version ${version}"
# The Makefile puts the tarball into our `dist` directory.
upload_raw_file_to_cloudsmith "dist/${UX_FILENAME}" "${version}"

artifact_json "${version}" "${UX_FILENAME}" > "${UX_ARTIFACTS_FILE}"

echo "--- :buildkite: Uploading ${UX_ARTIFACTS_FILE} file"
buildkite-agent artifact upload "${UX_ARTIFACTS_FILE}"
