cp -r "../sqs-microservice"  . &&
cp -r "../graph-descriptions" . &&
cp -r "../graph-generator-lib"  . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t 58c6c63dcf52 cargo build --release &&
cp "./target/x86_64-unknown-linux-musl/release/sysmon-subgraph-generator" . &&
zip "./sysmon-subgraph-generator.zip" "./sysmon-subgraph-generator" &&
cp "./sysmon-subgraph-generator.zip" "../grapl-cdk/"
rm "./sysmon-subgraph-generator.zip"
rm "./sysmon-subgraph-generator"
