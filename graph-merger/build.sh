#!/usr/bin/env bash
docker run --rm -it -v "$(pwd)":/home/rust/src -t c919a042d4d5 cargo build --release &&
cp "./target/x86_64-unknown-linux-musl/release/graph-merger" "./bootstrap" &&
zip "./graph-merger.zip" "./bootstrap" &&
cp "./graph-merger.zip" "../grapl-cdk/"
rm "./bootstrap"
rm ./graph-merger.zip
date
