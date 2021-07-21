python_library(
    sources=["*.py"]
)

python_awslambda(
    name="lambda",
    output_path="e2e-test-runner-lambda.zip",
    runtime="python3.7",
    handler="./main.py:lambda_handler",
    dependencies=[":e2e_test_runner"]
)
