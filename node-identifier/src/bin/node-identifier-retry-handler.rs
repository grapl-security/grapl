extern crate lambda_runtime as lambda;
extern crate node_identifier;
extern crate openssl_probe;
extern crate simple_logger;

use node_identifier::retry_handler;

use lambda::lambda;

fn main() {
    openssl_probe::init_ssl_cert_env_vars();
    simple_logger::init_with_level(log::Level::Info).unwrap();
    lambda!(retry_handler);
}
