python_library(
    name="graplctl_lib",
    skip_mypy=True,
    dependencies=[
        'src/python/grapl-common:grapl-common',
    ],
)

pex_binary(
    entry_point="cli.py:main",
    zip_safe=False,
)
