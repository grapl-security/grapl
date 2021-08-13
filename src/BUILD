# This has to be up at the top level because of how Pants figures out
# which code belongs to which python_distributions in order to sort
# out dependencies properly. Since this depends on both Python code
# and protobuf code, it has to be "higher" than both of them.
grapl_python_distribution(
    name = "graplinc_python_protobuf_stubs",
    dependencies = [
        "src/proto:graplinc",
    ],
    provides = setup_py(
        name = "graplinc-grapl-api",
        readme = "src/proto/graplinc/README.python-codegen.md",
        version = "0.1.1",
        description = "Protobuf code for interacting with the Grapl API",
    )
)
