#!/bin/bash

echo "deploying dgraph"
sudo su ec2-user
cd $HOME
export GRAPL_DEPLOY_NAME=$1
export AWS_LOGS_GROUP="$GRAPL_DEPLOY_NAME-grapl-dgraph"
export AWS01_NAME=$2
export AWS02_NAME=$3
export AWS03_NAME=$4

aws s3 cp s3://$GRAPL_DEPLOY_NAME-dgraph-config-bucket/docker-compose-dgraph.yml .
aws s3 cp s3://$GRAPL_DEPLOY_NAME-dgraph-config-bucket/envoy.yaml .

docker stack deploy -c docker-compose-dgraph.yml dgraph
echo "deployed dgraph"
