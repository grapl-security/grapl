#!/bin/bash

# No unset vars please
set -o nounset

if [ -z "$VERSION" ]
then
  echo "Please set VERSION="
  exit 1
fi

# Directory containing this shell script
CDK_DIR=$(realpath $(dirname "$0"))
cd $CDK_DIR
GRAPLROOT=$(realpath ../../..)
ZIPS_DIR="$CDK_DIR/zips"

function getzip_a() {
    echo "Zipping $1"
    # Make a temp dir with './boostrap' in it
    TEMPDIR="/tmp/zipped_$1"
    mkdir $TEMPDIR
    cd $TEMPDIR
    cp "$GRAPLROOT/dist/$1" ./bootstrap
    # zip it up into CDK/zips
    zip -9 --quiet --display-globaldots "$ZIPS_DIR/$1-$VERSION.zip" ./bootstrap
    # Go back home, clean up 
    cd $CDK_DIR
    rm -r $TEMPDIR
    echo "Done zipping $1"
}

function getzip_b() {
    echo "Copying $1"
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

echo "Done extracting artifacts"
