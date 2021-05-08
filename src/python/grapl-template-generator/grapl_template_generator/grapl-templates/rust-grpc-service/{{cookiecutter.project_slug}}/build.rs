fn main() {
    let config = tonic_build::configure();

    config
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "{{cookiecutter.proto_path}}{{cookiecutter.snake_project_name}}.proto",
            ],
            &["{{cookiecutter.proto_path}}"],
        )
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
