python_sources(
    name="analyzer_executor",
    dependencies=[
        "./analyzer_executor_lib",
        "./plugins",
    ],
)

pex_binary(
    output_path="analyzer-executor.pex",
    entry_point="./run.py:run",
)
