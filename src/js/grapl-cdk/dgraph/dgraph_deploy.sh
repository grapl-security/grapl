#!/bin/bash

set -euo pipefail

echo "deploying dgraph"
export GRAPL_DEPLOY_NAME="$1"
sudo -u ec2-user -- aws s3 cp "s3://${GRAPL_DEPLOY_NAME}-dgraph-config-bucket/docker-compose-dgraph.yml" ~ec2-user/
sudo -u ec2-user -- aws s3 cp "s3://${GRAPL_DEPLOY_NAME}-dgraph-config-bucket/envoy.yaml" ~ec2-user/

sudo -u ec2-user \
    --login \
    PWD=~ec2-user \
    AWS01_NAME="$2" \
    AWS02_NAME="$3" \
    AWS03_NAME="$4" \
    AWS_LOGS_GROUP="${GRAPL_DEPLOY_NAME}-grapl-dgraph" \
    -- docker stack deploy -c docker-compose-dgraph.yml dgraph

echo "deployed dgraph"
