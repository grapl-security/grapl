#!/bin/bash
cd ./build/ || return

export AWS_ACCESS_KEY_ID=minioadmin
export AWS_SECRET_ACCESS_KEY=minioadmin


aws s3 sync . s3://local-grapl-engagement-ux-bucket/ --endpoint-url=http://localhost:9000 --region=us-east-1
