def grapl_python_distribution(**kwargs):
    """Convenience macro to build a source distribution and wheel by default for a Python distribution.

    This is just to introduce consistency and cut down on a bit of extra typing.
    """

    kwargs["wheel"] = True
    kwargs["sdist"] = True

    python_distribution(**kwargs)


def py_typed(**kwargs):
    """Creates a `resources` target for the `py.typed` file in this directory."""
    if "name" not in kwargs:
        kwargs["name"] = "py_typed"

    if "sources" not in kwargs:
        kwargs["sources"] = ["py.typed"]

    resources(**kwargs)
