import json
import os
from dataclasses import dataclass, field
from typing import Mapping, Optional, Union

import pulumi_aws as aws
from infra.metric_forwarder import MetricForwarder
from infra.util import (
    DEPLOYMENT_NAME,
    GLOBAL_LAMBDA_ZIP_TAG,
    IS_LOCAL,
    import_aware_opts,
)
from typing_extensions import Literal

import pulumi


def code_path_for(lambda_fn: str) -> str:
    """Given the name of a lambda, return the local path of the ZIP archive for that lambda.

    Looks in "<REPOSITORY_ROOT>/src/js/grapl-cdk/zips" currently, but
    this can be overridden by setting the `GRAPL_LAMBDA_ZIP_DIR`
    environment variable to an appropriate directory.

    Uses the globally-defined "tag" (see `GLOBAL_LAMBDA_ZIP_TAG`) to
    put together the appropriate file name.

    """
    root_dir = os.getenv("GRAPL_LAMBDA_ZIP_DIR", "../src/js/grapl-cdk/zips")
    return f"{root_dir}/{lambda_fn}-{GLOBAL_LAMBDA_ZIP_TAG}.zip"


LambdaPackageType = Literal["Zip", "Image"]


@dataclass(frozen=True)
class LambdaArgs:
    """Encapsulates all the arguments for defining and running a lambda function.

    In general, you should probably use a subclass of this to ensure
    consistent configuration whenever possible.

    """

    execution_role: aws.iam.Role
    """ The IAM Role the function will execute as."""

    handler: str
    """ The entrypoint for the function. """

    code_path: str
    """ The path to a local file on disk that contains the code for this
    function."""

    description: str
    """ Textual description of what this function does. """

    runtime: aws.lambda_.Runtime
    """ The lambda runtime to use for this function. """

    package_type: LambdaPackageType
    """ The format of the lambda package. """

    memory_size: int
    """ How many megabytes of memory this function will use. """

    timeout: int
    """ How many seconds this function has to execute. """

    env: Mapping[str, Union[str, pulumi.Output[str]]]
    """ Environment variables to set for each function invocation. """


@dataclass(frozen=True)
class PythonLambdaArgs(LambdaArgs):
    """ Arguments for instantiating a Python ZIP lambda function. """

    runtime: aws.lambda_.Runtime = aws.lambda_.Runtime.PYTHON3D7
    package_type: LambdaPackageType = "Zip"
    memory_size: int = 128
    timeout: int = 45
    env: Mapping[str, Union[str, pulumi.Output[str]]] = field(default_factory=dict)


class LambdaExecutionRole(aws.iam.Role):
    """An IAM role dedicated to executing a specific lambda function.

    It may be augmented by adding additional `RolePolicy` documents
    later, but provides a consistent base for all lambda-related roles
    to derive from.

    """

    def __init__(
        self, name: str, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:
        super().__init__(
            f"{name}-execution-role",
            description=f"Lambda execution role for {name}",
            assume_role_policy=json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Action": "sts:AssumeRole",
                            "Principal": {"Service": "lambda.amazonaws.com"},
                        }
                    ],
                }
            ),
            # TODO: We want to remove managed policies eventually
            # https://github.com/grapl-security/issue-tracker/issues/106
            managed_policy_arns=[
                "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole",
                "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole",
            ],
            opts=opts,
        )


class Lambda(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        args: LambdaArgs,
        forwarder: Optional[MetricForwarder] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:Lambda", name, None, opts)

        # TODO:
        # - Add VPC to lambda

        lambda_name = f"{DEPLOYMENT_NAME}-{name}"
        self.function = aws.lambda_.Function(
            f"{name}-lambda",
            name=name,
            description=args.description,
            runtime=args.runtime,
            package_type=args.package_type,
            handler=args.handler,
            code=pulumi.FileArchive(args.code_path),
            environment=aws.lambda_.FunctionEnvironmentArgs(variables=args.env),
            memory_size=args.memory_size,
            timeout=args.timeout,
            role=args.execution_role.arn,
            opts=import_aware_opts(lambda_name, parent=self),
        )

        if not IS_LOCAL:
            # TODO: While Localstack can create lambda aliases just
            # fine, it doesn't appear to be able to *delete* them
            # right now. Since we don't use aliases in Local Grapl,
            # we'll just not create them in order to facilitate local
            # testing of the overall deployment logic (which often
            # involves repeated `pulumi up`/`pulumi destroy` cycles
            self.alias = aws.lambda_.Alias(
                f"{name}-live-alias",
                function_name=self.function.arn,
                function_version=self.function.version,
                name="live",
                opts=import_aware_opts(f"{lambda_name}/live", parent=self),
            )

        if forwarder:

            # Lambda function output is automatically sent to log
            # groups named like this; we create one explicitly for
            # integration with the metric forwarder.
            self.log_group = aws.cloudwatch.LogGroup(
                f"{name}-log-group",
                name=f"/aws/lambda/{lambda_name}",
                opts=pulumi.ResourceOptions(parent=self),
            )

            # Allow the metric forwarder to be invoked with by this
            # lambda function's log group.
            self.forwarder_permission = aws.lambda_.Permission(
                f"{name}-log-group-invokes-metric-forwarder",
                principal=f"logs.amazonaws.com",
                action="lambda:InvokeFunction",
                function=forwarder.function.function.arn,
                source_arn=self.log_group.arn.apply(lambda arn: f"{arn}:*"),
                opts=pulumi.ResourceOptions(parent=self),
            )

            # Then, with that permission granted, configure the metric
            # forwarder to receive all 'MONITORING' log messages from
            # the lambda function.
            self.forwarder_subscription_filter = aws.cloudwatch.LogSubscriptionFilter(
                f"metric-forwarder-subscribes-to-{name}-monitoring-logs",
                log_group=self.log_group.name,
                filter_pattern="MONITORING",
                destination_arn=forwarder.function.function.arn,
                opts=pulumi.ResourceOptions(
                    depends_on=[self.forwarder_permission], parent=self
                ),
            )

        self.register_outputs({})
