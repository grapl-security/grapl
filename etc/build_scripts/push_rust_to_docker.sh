#!/bin/bash
set -eu

services=(
    "sysmon-subgraph-generator"
    "osquery-subgraph-generator"
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
