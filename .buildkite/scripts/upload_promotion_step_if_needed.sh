#!/usr/bin/env bash

set -euo pipefail

if [ "$(jq 'length' all_artifacts.json)" == "0" ]; then
    echo "No artifacts to promote!"
else
    echo "We have at least one artifact to promote"
    buildkite-agent pipeline upload .buildkite/pipeline.merge.promote.yml
fi
