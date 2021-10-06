python_library(
    sources=["*.py"]
)

pex_binary(
    name="pex",
    output_path="provisioner.pex",
    entry_point="./app.py:provision",
    execution_mode="venv",
)