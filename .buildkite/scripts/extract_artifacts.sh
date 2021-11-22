#!/usr/bin/env bash

set -euo pipefail

readonly stack="${1}"

pulumi login
pulumi config get artifacts \
    --cwd=pulumi/grapl \
    --stack="${stack}" \
    --json |
    jq '.objectValue' > current_artifacts.json

# We have to unset the AWS credentials injected by the
# asume-role plugin if we're going to subsequently upload the
# file to our bucket :(
unset AWS_ACCESS_KEY_ID
unset AWS_SECRET_ACCESS_KEY
unset AWS_SESSION_TOKEN

buildkite-agent artifact upload current_artifacts.json
