python_sources(name="test_files", sources=["*.py"])

pex_binary(
    name="pex",
    output_path="e2e-tests.pex",
    entry_point="./main.py:main",
    # Explicit `dependencies` needed, because `pytest` picks up tests
    # dynamically, not by explicit imports
    dependencies=[":test_files"],
    tags=["e2e-test-pex"],
)
