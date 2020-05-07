#!/usr/bin/env bash
docker run --rm -it -v "$(pwd)/../grapl-config":/home/rust/grapl-config -v "$(pwd)":/home/rust/src -t grapl/grapl_rust_base cargo build --release &&
cp "./target/x86_64-unknown-linux-musl/release/sysmon-subgraph-generator" "./sysmon-subgraph-generator" &&
cp "./sysmon-subgraph-generator" "./bootstrap" &&
zip "./sysmon-subgraph-generator.zip" "./bootstrap" &&
cp "./sysmon-subgraph-generator.zip" "../grapl-cdk/"
rm "./sysmon-subgraph-generator.zip"
rm "./bootstrap"
date
