#!/bin/bash

function getzip_a() {
    docker create -ti --name "cp-$1" "grapl/grapl-$1"
    docker cp "cp-$1:/$1" ./bootstrap
    docker rm -f "cp-$1"
    zip -9 "zips/$1-$VERSION.zip" ./bootstrap
    rm ./bootstrap
}

function getzip_b() {
    docker create -ti --name "cp-$1" "grapl/grapl-$1"
    docker cp "cp-$1:/home/grapl/lambda.zip" "zips/$1-$VERSION.zip"
    docker rm -f "cp-$1"
}

function getzip_c() {
    docker create -ti --name "cp-$1" "grapl/grapl-$1"
    docker cp "cp-$1:/lambda.zip" "zips/$1-$VERSION.zip"
    docker rm -f "cp-$1"
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
)

cs=(
    "graphql-endpoint"
)

for a in "${as[@]}"; do
    getzip_a $a
done

for b in "${bs[@]}"; do
    getzip_b $b
done

for c in "${cs[@]}"; do
    getzip_c $c
done
