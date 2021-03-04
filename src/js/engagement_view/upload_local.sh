#!/bin/bash
cd ./build/ || exit

set -xeuo pipefail

export AWS_ACCESS_KEY_ID="${S3_ACCESS_KEY_ID}"
export AWS_SECRET_ACCESS_KEY="${S3_ACCESS_KEY_SECRET}"

aws s3 sync . s3://local-grapl-engagement-ux-bucket/ \
    --endpoint-url="${S3_ENDPOINT}" --region="${AWS_REGION}"
