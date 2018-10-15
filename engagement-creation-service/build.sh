#!/usr/bin/env bash
cp -r "../graph-descriptions" . &&
cp -r "../incident-graph" . &&
cp -r "../sqs-microservice"  . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t 3b07546503c6 cargo build --release &&
cp "./target/x86_64-unknown-linux-musl/release/engagement-creation-service" . &&
zip "./engagement-creation-service.zip" "./engagement-creation-service" &&
cp "./engagement-creation-service.zip" "../grapl-cdk/" &&
rm "./engagement-creation-service"
rm "./engagement-creation-service.zip"


