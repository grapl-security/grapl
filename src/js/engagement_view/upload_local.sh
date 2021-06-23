#!/bin/bash
cd ./build/ || exit

set -xeuo pipefail

aws s3 sync . s3://local-grapl-engagement-ux-bucket/ \
    --endpoint-url="${AWS_ENDPOINT}" --region="${AWS_REGION}"
