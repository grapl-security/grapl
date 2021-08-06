pub mod server;
pub mod client;

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
