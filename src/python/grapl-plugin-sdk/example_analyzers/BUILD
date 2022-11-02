python_sources(
    name="example_analyzers_lib",
)

pex_binary(
    name="suspicious_svchost",
    output_path="suspicious_svchost_analyzer.pex",  # where it ends up in dist/
    entry_point="suspicious_svchost.py:main",
    tags=["test-fixture-pex"],
)
