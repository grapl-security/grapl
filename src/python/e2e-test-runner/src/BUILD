python_library(
    sources=["*.py"]
)

python_awslambda(
    name="lambda",
    runtime="python3.7",
    handler="./main.py:main",
    dependencies=["./test_main.py", "./schemas.py"]
)
