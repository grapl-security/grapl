#!/usr/bin/env bash

# Given a series of names of artifact files (i.e., files containing
# flat JSON objects mapping artifact names to versions), retrieve them
# from the Buildkite artifact storage facility, merge them all into a
# single new artifact file, and then upload that for subsequent
# processing.
#
# This will allow this logic to be reused across multiple pipelines,
# regardless of how many artifact files they may generate.

set -euo pipefail

source .buildkite/scripts/lib/artifacts.sh

all_files=("$@")

for file in "${all_files[@]}"; do
    download_artifact_file "${file}"
done

merge_artifact_files > "${ALL_ARTIFACTS_JSON_FILE}"

echo "--- :buildkite: Uploading ${ALL_ARTIFACTS_JSON_FILE} file"
buildkite-agent artifact upload "${ALL_ARTIFACTS_JSON_FILE}"
