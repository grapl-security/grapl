#!/usr/bin/env bash

set -euo pipefail

readonly stack="${1}"

pulumi login
pulumi config get artifacts \
    --cwd=pulumi/grapl \
    --stack="${stack}" \
    --json |
    jq '.objectValue' > current_artifacts.json

buildkite-agent artifact upload current_artifacts.json
