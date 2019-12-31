#!/usr/bin/env bash
docker run --rm -it -v "$(pwd)":/home/rust/src -t 096f585a5019 cargo build --release &&
cp "./target/x86_64-unknown-linux-musl/release/sysmon-subgraph-generator" "./bootstrap" &&
zip "./sysmon-subgraph-generator.zip" "./bootstrap" &&
cp "./sysmon-subgraph-generator.zip" "../grapl-cdk/"
rm "./sysmon-subgraph-generator.zip"
rm "./bootstrap"
date
