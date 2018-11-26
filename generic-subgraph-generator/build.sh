cp -r "../sqs-microservice"  . &&
cp -r "../graph-descriptions" . &&
cp -r "../graph-generator-lib"  . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t ea24bf58caa2 cargo build --release --bin generic-subgraph-generator &&
cp "./target/x86_64-unknown-linux-musl/release/generic-subgraph-generator" . &&
zip "./generic-subgraph-generator.zip" "./generic-subgraph-generator" &&
cp "./generic-subgraph-generator.zip" "../grapl-cdk/"
rm "./generic-subgraph-generator.zip"
rm "./generic-subgraph-generator"
