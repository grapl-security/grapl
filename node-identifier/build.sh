cp -r "../graph-descriptions" . &&
cp -r "../sqs-microservice"  . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t ea24bf58caa2 cargo build --release --bin node-identifier &&
cp "./target/x86_64-unknown-linux-musl/release/node-identifier" . &&
zip "./node-identifier.zip" "./node-identifier" &&
cp "./node-identifier.zip" "../grapl-cdk/"

rm ./node-identifier
rm ./node-identifier.zip



docker run --rm -it -v "$(pwd)":/home/rust/src -t ea24bf58caa2 cargo build --release --bin node-identifier-retry-handler &&
cp "./target/x86_64-unknown-linux-musl/release/node-identifier-retry-handler" . &&
zip "./node-identifier-retry-handler.zip" "./node-identifier-retry-handler" &&
cp "./node-identifier-retry-handler.zip" "../grapl-cdk/"


rm ./node-identifier-retry-handler
rm ./node-identifier-retry-handler.zip
