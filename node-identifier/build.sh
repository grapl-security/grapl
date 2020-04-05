docker run --rm -it -v "$(pwd)":/home/rust/src -t grapl/grapl_rust_base cargo build --release --bin node-identifier &&
cp "./target/x86_64-unknown-linux-musl/release/node-identifier" "./bootstrap" &&
cp "./target/x86_64-unknown-linux-musl/release/node-identifier" "./node-identifier" &&

zip "./node-identifier.zip" "./bootstrap" &&
cp "./node-identifier.zip" "../grapl-cdk/"

rm ./bootstrap
rm ./node-identifier.zip



docker run --rm -it -v "$(pwd)":/home/rust/src -t grapl/grapl_rust_base cargo build --release --bin node-identifier-retry-handler &&
cp "./target/x86_64-unknown-linux-musl/release/node-identifier-retry-handler" "./bootstrap" &&
cp "./target/x86_64-unknown-linux-musl/release/node-identifier-retry-handler" "./node-identifier-retry-handler" &&

zip "./node-identifier-retry-handler.zip" "./bootstrap" &&
cp "./node-identifier-retry-handler.zip" "../grapl-cdk/"


rm ./bootstrap
rm ./node-identifier-retry-handler.zip
date
