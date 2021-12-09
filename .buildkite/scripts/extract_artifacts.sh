#!/usr/bin/env bash

set -euo pipefail

readonly stack="${1}"

pulumi login
pulumi config get artifacts \
    --cwd=pulumi/grapl \
    --stack="${stack}" \
    --json |
    jq '.objectValue' > current_artifacts.json

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/unset_aws_creds_for_artifact_upload.sh"

buildkite-agent artifact upload current_artifacts.json
