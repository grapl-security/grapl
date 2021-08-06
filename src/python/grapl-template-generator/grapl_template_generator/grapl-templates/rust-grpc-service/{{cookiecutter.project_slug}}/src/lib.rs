/*
This module just re-exports the server, client, and protos. No need to modify.
*/

pub mod server;
pub mod client;

// In the future, this will be in rust-proto.
pub mod {{cookiecutter.snake_project_name}} {
    tonic::include_proto!("{{cookiecutter.snake_project_name}}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
	todo!("Write some tests!")
    }
}
