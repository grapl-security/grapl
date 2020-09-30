#!/bin/bash
set -u

services=(
    "sysmon-subgraph-generator"
    "generic-subgraph-generator"
    "node-identifier"
    "node-identifier-retry-handler"
    "graph-merger"
    "analyzer-dispatcher"
    "metric-forwarder"
)
for svc in "${services[@]}"; do
     docker push grapl/grapl-$svc:$TAG
done
