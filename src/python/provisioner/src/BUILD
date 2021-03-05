python_library(
    name="provisioner",
    sources=["app.py"],
    dependencies=[
        "3rdparty/boto3",
        "3rdparty/chalice",
        "3rdparty/mypy_boto3_dynamodb",
        "3rdparty/mypy_boto3_secretsmanager",
        "grapl_analyzerlib",
    ]
)

python_awslambda(
    name="lambda",
    runtime="python3.7",
    handler="app.py:provision",
)
