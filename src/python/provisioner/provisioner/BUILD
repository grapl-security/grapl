python_library(
    sources=["*.py"]
)

python_awslambda(
    name="lambda",
    output_path="provisioner-lambda.zip",
    runtime="python3.7",
    handler="./app.py:provision",
)
