from typing import Any  # todo: the aws-lambda-typing package also has type annotations for responses
import aws_lambda_typing as lambda_typing

# The `event` value's type depends on which event source you've subscribed to,
# check out 'lambda_typing.events'
def {{cookiecutter.lambda_handler}}(event: Any, context: lambda_typing.Context) -> Any:
    raise NotImplementedError
