#!/bin/bash
set -u

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
    cp dist/$svc ./bootstrap
    zip -9 $svc-$TAG.zip ./bootstrap
    rm ./bootstrap
    echo "::set-output name=$svc::$svc-$TAG.zip"
done
