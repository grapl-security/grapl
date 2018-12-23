extern crate node_identifier;
extern crate lambda_runtime as lambda;

use node_identifier::handler;

use lambda::lambda;

fn main() {
    lambda!(handler);
}
