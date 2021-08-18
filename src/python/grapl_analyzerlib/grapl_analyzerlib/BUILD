python_library(
    dependencies=[
        # TODO: Ideally, this would be picked up by Pants' automatic
        # dependency resolution, but something appears to be off with
        # it.
        "src/proto:graplinc"
        "src/python/python-proto:python_proto",
    ],
    skip_mypy=True
)

py_typed()
