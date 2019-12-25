extern crate lambda_runtime as lambda;
extern crate node_identifier;
extern crate simple_logger;

use node_identifier::retry_handler;

use lambda::lambda;

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    lambda!(retry_handler);
}
