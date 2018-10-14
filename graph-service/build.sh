cp -r "../graph-descriptions" . &&
cp -r "../sqs-microservice"  . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t 3b07546503c6 cargo build --release --bin graph-merger &&
cp "./target/x86_64-unknown-linux-musl/release/graph-merger" . &&
zip "./graph-merger.zip" "./graph-merger" &&
cp "./graph-merger.zip" "../grapl-cdk/"
rm ./graph-merger
rm ./graph-merger.zip
