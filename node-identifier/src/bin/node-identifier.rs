extern crate lambda_runtime as lambda;
extern crate node_identifier;
extern crate simple_logger;

use node_identifier::handler;

use lambda::lambda;

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    lambda!(handler);
}
