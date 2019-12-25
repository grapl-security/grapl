docker run --rm -it -v "$(pwd)":/home/rust/src -t 096f585a5019 cargo build --release --bin node-identifier &&
cp "./target/x86_64-unknown-linux-musl/release/node-identifier" "./bootstrap" &&
zip "./node-identifier.zip" "./bootstrap" &&
cp "./node-identifier.zip" "../grapl-cdk/"

rm ./bootstrap
rm ./node-identifier.zip



docker run --rm -it -v "$(pwd)":/home/rust/src -t 096f585a5019 cargo build --release --bin node-identifier-retry-handler &&
cp "./target/x86_64-unknown-linux-musl/release/node-identifier-retry-handler" "./bootstrap" &&
zip "./node-identifier-retry-handler.zip" "./bootstrap" &&
cp "./node-identifier-retry-handler.zip" "../grapl-cdk/"


rm ./bootstrap
rm ./node-identifier-retry-handler.zip
date
