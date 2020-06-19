#!/bin/bash

function getzip_a() {
    docker create -ti --name "cp-$1" "grapl/grapl-$1"
    docker cp "cp-$1:/$1" ./bootstrap
    docker rm -f "cp-$1"
    zip -9 "$1-$VERSION-$CHANNEL.zip" ./bootstrap
    rm ./bootstrap
}

function getzip_b() {
    docker create -ti --name "cp-$1" "grapl/grapl-$1"
    docker cp "cp-$1:/home/grapl/lambda.zip" "$1-$VERSION-$CHANNEL.zip"
    docker rm -f "cp-$1"
}

function getzip_c() {
    docker create -ti --name "cp-$1" "grapl/grapl-$1"
    docker cp "cp-$1:/lambda.zip" "$1-$VERSION-$CHANNEL.zip"
    docker rm -f "cp-$1"
}

builds=(
    "node-identifier"
    "sysmon-subgraph-generator"
    "generic-subgraph-generator"
    "node-identifier-retry-handler"
    "graph-merger"
    "analyzer-dispatcher"
)

asdf=(
    "analyzer-executor"
    "engagement-creator"
    "engagement-edge"
    "model-plugin-deployer"
    "dgraph-ttl"
)

cs=(
    "graphql-endpoint"
)

for b in "${builds[@]}"; do
    getzip_a $b
done

for a in "${asdf[@]}"; do
    getzip_b $a
done

for c in "${cs[@]}"; do
    getzip_c $c
done
