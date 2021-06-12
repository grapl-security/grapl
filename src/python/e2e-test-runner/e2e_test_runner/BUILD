python_library(
    sources=["*.py"]
)

python_awslambda(
    name="lambda",
    runtime="python3.7",
    handler="./main.py:lambda_handler",
    dependencies=["./test_main.py", "./schemas.py"]
)
