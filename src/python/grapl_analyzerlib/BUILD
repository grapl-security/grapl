grapl_python_distribution(
    dependencies = [
        "./grapl_analyzerlib",
        "./grapl_analyzerlib:py_typed",
        "./grapl_analyzerlib/nodes",
        "src/python/python-proto",
        # TODO: remove this once python-proto is sufficiently featureful
        "src:graplinc_python_protobuf_stubs",
    ],
    provides = setup_py(
        name = "grapl-analyzerlib",
        version = "0.2.73", # TODO: remove VERSION file? Or read from it?
        description = "Library for Grapl Analyzers"
    )
)
