def grapl_python_distribution(**kwargs):
    """ Convenience macro to build a source distribution and wheel by default for a Python distribution.

    If setup_py_commands are provided, they are used unchanged.

    This is just to introduce consistency and cut down on a bit of extra typing.
    """
    if "setup_py_commands" not in kwargs:
        kwargs["setup_py_commands"] = [ "sdist", "bdist_wheel" ]

    python_distribution(**kwargs)
