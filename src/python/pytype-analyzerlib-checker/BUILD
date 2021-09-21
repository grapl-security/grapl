python_library()

pex_binary(
    name="analyzerlib-checker",
    description="run pytype on the grapl analyzerlib",
    entry_point="pytype_analyzerlib_checker:main",
    dependencies=[
        "src/python/grapl_analyzerlib",
        "3rdparty/python:pytype",
    ],
    inherit_path="fallback",
)
