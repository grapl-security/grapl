#!/bin/bash
cd ./build/ || exit

export AWS_ACCESS_KEY_ID="test"
export AWS_SECRET_ACCESS_KEY="test"

aws s3 sync . s3://local-grapl-engagement-ux-bucket/ \
    --endpoint-url=http://localhost:4566 \
    --region=us-east-1
