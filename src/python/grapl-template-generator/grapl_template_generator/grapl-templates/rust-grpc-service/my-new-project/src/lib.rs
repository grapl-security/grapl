#![warn(clippy::all)]

pub mod server;
pub mod client;

pub mod my_new_project {
    tonic::include_proto!("my_new_project");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
	todo!("Write some tests!")
    }
}
