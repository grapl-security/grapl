#!/usr/bin/env bash
cp -r "../sqs-microservice"  . &&
cp -r "../graph-descriptions" . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t ea24bf58caa2 cargo build --release --bin node-identity-mapper &&
cp "./target/x86_64-unknown-linux-musl/release/node-identity-mapper" . &&
zip "./node-identity-mapper.zip" "./node-identity-mapper" &&
cp "./node-identity-mapper.zip" "../grapl-cdk/"
rm "./node-identity-mapper.zip"
rm "./node-identity-mapper"
