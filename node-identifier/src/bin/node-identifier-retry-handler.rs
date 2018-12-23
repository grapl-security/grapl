extern crate node_identifier;
extern crate lambda_runtime as lambda;

use node_identifier::retry_handler;

use lambda::lambda;

fn main() {
    lambda!(retry_handler);
}
