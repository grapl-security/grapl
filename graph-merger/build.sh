cp -r "../graph-descriptions" . &&
cp -r "../sqs-microservice"  . &&
docker run --rm -it -v "$(pwd)":/home/rust/src -t 58c6c63dcf52 cargo build --release &&
cp "./target/x86_64-unknown-linux-musl/release/graph-merger" . &&
zip "./graph-merger.zip" "./graph-merger" &&
cp "./graph-merger.zip" "../grapl-cdk/"
rm ./graph-merger
rm ./graph-merger.zip
