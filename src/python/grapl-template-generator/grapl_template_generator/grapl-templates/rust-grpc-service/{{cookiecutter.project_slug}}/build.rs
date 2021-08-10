fn main() {
    let config = tonic_build::configure();

    config
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "../../proto/graplinc/grapl/api/{{cookiecutter.snake_project_name}}/{{cookiecutter.snake_project_name}}.proto",
                "../../proto/graplinc/grapl/api/{{cookiecutter.snake_project_name}}/{{cookiecutter.snake_project_name}}_health.proto",
            ],
            &["../../proto/"],
        )
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
