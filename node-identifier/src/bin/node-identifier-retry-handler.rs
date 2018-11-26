extern crate node_identifier;

use node_identifier::identify_nodes;


fn main() {
    identify_nodes(true);
}
//rust-musl-builder cargo build --release && cp ./target/x86_64-unknown-linux-musl/release/node-identifier . && zip ./node-identifier.zip ./node-identifier && cp ./node-identifier.zip ~/workspace/grapl/grapl-cdk/ && rm ./node-identifier.zip
