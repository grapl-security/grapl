python_library(
    dependencies=[
        # TODO: Ideally, this would be picked up by Pants' automatic
        # dependency resolution, but something appears to be off with
        # it.
        "src/python/graplinc/graplinc/grapl/api"
    ]
)

# TODO: Consider a plugin macro for this?
resources(
    name = "typehints",
    sources = ["py.typed"]
)