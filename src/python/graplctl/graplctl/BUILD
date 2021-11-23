python_sources(
    name="graplctl_lib",
    dependencies=[
        "src/python/grapl-common:grapl-common",
    ],
)

pex_binary(
    entry_point="cli.py:main",
    # TODO: Once https://github.com/pantsbuild/pants/pull/13690 lands,
    # remove this work-around
    # See also https://pantsbuild.slack.com/archives/C046T6T9U/p1637597807103900
    dependencies=[
        "!!src/python/grapl-common:grapl-common"
    ]
)
