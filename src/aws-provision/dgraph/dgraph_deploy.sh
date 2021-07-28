#!/bin/bash

set -euo pipefail

echo "--- Deploying dgraph ---"
readonly grapl_deploy_name="$1"
readonly manager="${2}"
readonly worker1="${3}"
readonly worker2="${4}"

sudo -u ec2-user -- aws s3 cp "s3://${grapl_deploy_name}-dgraph-config-bucket/docker-compose-dgraph.yml" ~ec2-user/
sudo -u ec2-user -- aws s3 cp "s3://${grapl_deploy_name}-dgraph-config-bucket/envoy.yaml" ~ec2-user/

sudo -u ec2-user \
    --login \
    PWD=~ec2-user \
    AWS01_NAME="${manager}" \
    AWS02_NAME="${worker1}" \
    AWS03_NAME="${worker2}" \
    AWS_LOGS_GROUP="${grapl_deploy_name}-grapl-dgraph" \
    -- docker stack deploy -c docker-compose-dgraph.yml dgraph

echo "--- Deployed dgraph ---"

echo "Issues with one of the services? To debug, SSM into ${manager} and run:"
echo "sudo -u ec2-user --login docker service logs dgraph_alpha1 (or any other service)"
