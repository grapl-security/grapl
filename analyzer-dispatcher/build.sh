#!/usr/bin/env bash
cp -r "../graph-descriptions" . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t 58c6c63dcf52 cargo build --release &&
cp "./target/x86_64-unknown-linux-musl/release/analyzer-dispatcher" "./bootstrap" &&
zip "./analyzer-dispatcher.zip" "./bootstrap" &&
cp "./analyzer-dispatcher.zip" "../grapl-cdk/"
rm "./bootstrap"
rm ./analyzer-dispatcher.zip
date
