#!/bin/bash
cd ./build/ || exit

set -xeuo pipefail

aws s3 sync . s3://local-grapl-engagement-ux-bucket/ \
    --endpoint-url="${GRAPL_AWS_ENDPOINT}" --region="${AWS_REGION}"
