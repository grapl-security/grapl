#!/usr/bin/env bash
cp -r "../graph-descriptions" . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t 58c6c63dcf52 cargo build --release &&
cp "./target/x86_64-unknown-linux-musl/release/node-identity-mapper" "./bootstrap" &&
zip "./node-identity-mapper.zip" "./bootstrap" &&
cp "./node-identity-mapper.zip" "../grapl-cdk/"
rm "./bootstrap"
rm ./node-identity-mapper.zip
date
