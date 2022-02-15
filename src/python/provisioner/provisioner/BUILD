python_sources(sources=["*.py"])

pex_binary(
    name="pex",
    output_path="provisioner.pex",
    entry_point="./app.py:provision",
    tags=["service-pex"],
)
