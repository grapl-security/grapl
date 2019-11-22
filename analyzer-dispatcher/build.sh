#!/usr/bin/env bash
docker run --rm -it -v "$(pwd)":/home/rust/src df6136cbfcc5 cargo build --release &&
cp "./target/x86_64-unknown-linux-musl/release/analyzer-dispatcher" "./bootstrap" &&
zip "./analyzer-dispatcher.zip" "./bootstrap" &&
cp "./analyzer-dispatcher.zip" "../grapl-cdk/"
rm "./bootstrap"
rm ./analyzer-dispatcher.zip
date
