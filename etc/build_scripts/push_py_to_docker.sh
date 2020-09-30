#!/bin/bash
set -u

services=(
    analyzer-executor
    engagement-creator
    notebook
    engagement-edge
    model-plugin-deployer
    graph-provision
    dynamodb-provision
    dgraph-ttl
)

for svc in "${services[@]}"; do
    docker push grapl/grapl-$svc:$TAG
done
