#!/usr/bin/env bash
cp -r "../graph-descriptions" . &&
cp -r "../graph-generator-lib"  . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t 58c6c63dcf52 cargo build --release --bin generic-subgraph-generator &&
cp "./target/x86_64-unknown-linux-musl/release/generic-subgraph-generator" "./bootstrap" &&
zip "./generic-subgraph-generator.zip" "./bootstrap" &&
cp "./generic-subgraph-generator.zip" "../grapl-cdk/"
rm "./generic-subgraph-generator.zip"
rm "./bootstrap"
