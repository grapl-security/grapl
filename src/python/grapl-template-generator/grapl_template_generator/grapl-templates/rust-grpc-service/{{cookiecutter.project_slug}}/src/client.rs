pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.snake_project_name}}_rpc_client::{{cookiecutter.crate_name}}RpcClient;
pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.crate_name}}Request;

pub use tower::timeout::Timeout;
pub use tonic::transport::Channel;

/* 
If you want to provide a higher-level client abstraction - like 
a {{cookiecutter.crate_name}}Client that hides the grpc implementation details -
this would be the place to add that.
*/