#!/bin/bash

if [ -z "$VERSION" ]
then
  echo "Please set VERSION="
  exit 1
fi

export GRAPLROOT=$(realpath ../../..)
export ZIPS_DIR=$(realpath zips)

function getzip_a() {
    echo "Zipping $1"

    export TEMPDIR="/tmp/zipped_$1"
    mkdir $TEMPDIR
    cd $TEMPDIR

    cp "$GRAPLROOT/dist/$1" ./bootstrap
    zip -9 "$ZIPS_DIR/$1-$VERSION.zip" ./bootstrap
    cd -
    rm -r $TEMPDIR
    echo "Done zipping $1"
}

function getzip_b() {
    cp "$GRAPLROOT/dist/$1/lambda.zip" "$ZIPS_DIR/$1-$VERSION.zip"
}

as=(
    "node-identifier"
    "sysmon-subgraph-generator"
    "generic-subgraph-generator"
    "node-identifier-retry-handler"
    "graph-merger"
    "analyzer-dispatcher"
    "metric-forwarder"
)

bs=(
    "analyzer-executor"
    "engagement-creator"
    "engagement-edge"
    "model-plugin-deployer"
    "dgraph-ttl"
    "graphql-endpoint"
)

# Doing the zips in parallel brings it down from 2m37s to 58s
for a in "${as[@]}"; do
    getzip_a $a &
done
wait

for b in "${bs[@]}"; do
    getzip_b $b
done
