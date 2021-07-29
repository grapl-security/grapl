#!/bin/bash

set -euo pipefail

echo "--- Deploying dgraph ---"

readonly manager_hostname="${1}"
readonly worker_0_hostname="${2}"
readonly worker_1_hostname="${3}"
readonly dgraph_config_bucket="${4}"
readonly dgraph_logs_group="${5}"

sudo -u ec2-user -- aws s3 cp "s3://${dgraph_config_bucket}/docker-compose-dgraph.yml" ~ec2-user/
sudo -u ec2-user -- aws s3 cp "s3://${dgraph_config_bucket}/envoy.yaml" ~ec2-user/

sudo -u ec2-user \
    --login \
    PWD=~ec2-user \
    AWS01_NAME="${manager_hostname}" \
    AWS02_NAME="${worker_0_hostname}" \
    AWS03_NAME="${worker_1_hostname}" \
    AWS_LOGS_GROUP="${dgraph_logs_group}" \
    -- docker stack deploy -c docker-compose-dgraph.yml dgraph

echo "--- Deployed dgraph ---"

# To debug services that don't work, SSM into the manager and
# sudo -u ec2-user --login docker service logs dgraph_alpha1 (or any other service)
