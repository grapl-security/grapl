fn main() {
    let config = tonic_build::configure();

    config
        .build_server(true)
        .build_client(true)
        .compile_protos(
            // These protos are moved to this directory by `def move_protos_to_global_proto_dir`
            &[
                "../../proto/graplinc/grapl/api/{{cookiecutter.snake_project_name}}/{{cookiecutter.snake_project_name}}.proto",
                "../../proto/graplinc/grapl/api/{{cookiecutter.snake_project_name}}/{{cookiecutter.snake_project_name}}_health.proto",
            ],
            &["../../proto/"],
        )
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
