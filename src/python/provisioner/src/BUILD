python_library(
    name="provisioner",
    sources=["app.py"],
)

python_awslambda(
    name="lambda",
    runtime="python3.7",
    handler="app.py:provision",
)
