#!/bin/bash
set -u

services=(
    analyzer-executor
    engagement-creator
    notebook
    engagement-edge
    model-plugin-deployer
    provision
    dgraph-ttl
)

for svc in "${services[@]}"; do
    docker push grapl/grapl-$svc:$TAG
done
