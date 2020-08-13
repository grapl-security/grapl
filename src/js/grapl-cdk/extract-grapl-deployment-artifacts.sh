#!/bin/bash

function getzip_a() {
    cp "../../../dist/$1" ./bootstrap
    zip -9 "zips/$1-$VERSION.zip" ./bootstrap
    rm ./bootstrap
}

function getzip_b() {
    cp "../../../dist/$1/lambda.zip" "zips/$1-$VERSION.zip"
}

as=(
    "node-identifier"
    "sysmon-subgraph-generator"
    "generic-subgraph-generator"
    "node-identifier-retry-handler"
    "graph-merger"
    "analyzer-dispatcher"
)

bs=(
    "analyzer-executor"
    "engagement-creator"
    "engagement-edge"
    "model-plugin-deployer"
    "dgraph-ttl"
    "graphql-endpoint"
)

for a in "${as[@]}"; do
    getzip_a $a
done

for b in "${bs[@]}"; do
    getzip_b $b
done
