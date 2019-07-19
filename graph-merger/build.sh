#!/usr/bin/env bash
cp -r "../graph-descriptions" . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t df6136cbfcc5 cargo build --release &&
cp "./target/x86_64-unknown-linux-musl/release/graph-merger" "./bootstrap" &&
zip "./graph-merger.zip" "./bootstrap" &&
cp "./graph-merger.zip" "../grapl-cdk/"
rm "./bootstrap"
rm ./graph-merger.zip
date
