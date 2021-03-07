#!/bin/bash
cd ./build/ || exit

export AWS_ACCESS_KEY_ID="THIS_IS_A_FAKE_AWS_ACCESS_KEY_ID"
export AWS_SECRET_ACCESS_KEY="THIS_IS_A_FAKE_AWS_SECRET_ACCESS_KEY"

aws s3 sync . s3://local-grapl-engagement-ux-bucket/ \
    --endpoint-url=http://localhost:9000 \
    --region=us-east-1
