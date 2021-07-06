import json
import os
from dataclasses import dataclass, field
from typing import Mapping, Optional, Union

import pulumi_aws as aws
from infra.config import (
    DEPLOYMENT_NAME,
    LOCAL_GRAPL,
    SERVICE_LOG_RETENTION_DAYS,
    repository_path,
)
from infra.network import Network
from typing_extensions import Literal

import pulumi


class LambdaResolver:
    """Encapsulates the logic required to generate a `pulumi.Archive` for
    a given AWS Lambda function, whether from a local directory or
    from a remote Cloudsmith repository.
    """

    @staticmethod
    def _code_path_for(lambda_fn: str) -> str:
        """Given the name of a lambda, return the local path of the ZIP archive for that lambda.

        Looks in "<REPOSITORY_ROOT>/dist" currently, but
        this can be overridden by setting the `GRAPL_LAMBDA_ZIP_DIR`
        environment variable to an appropriate directory.
        """
        root_dir = os.getenv("GRAPL_LAMBDA_ZIP_DIR", repository_path("dist"))
        return f"{root_dir}/{lambda_fn}-lambda.zip"

    @staticmethod
    def _cloudsmith_url(lambda_fn: str, version: str, repository: str) -> str:
        # TODO: The repository should be configurable somehow (perhaps
        # by explicitly setting it in stack configuration).
        return f"https://dl.cloudsmith.io/public/grapl/{repository}/raw/versions/{version}/{lambda_fn}-lambda.zip"

    @staticmethod
    def resolve(lambda_fn: str) -> pulumi.Archive:
        """Resolve the code for a lambda function, either from a local directory or from our Cloudsmith repository.

        Downloading from Cloudsmith relies on a version being found for
        the given lambda function in the stack configuration "artifacts"
        object.
        """
        artifacts = pulumi.Config().get_object("artifacts") or {}
        version = artifacts.get(lambda_fn)
        if version:
            url = LambdaResolver._cloudsmith_url(lambda_fn, version, "raw")
            pulumi.info(f"Version found for {lambda_fn}: {version} ({url})")
            return pulumi.RemoteArchive(url)
        else:
            pulumi.info(f"Version NOT found for {lambda_fn}; using local file")
            return pulumi.FileArchive(LambdaResolver._code_path_for(lambda_fn))


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

    code: pulumi.Archive
    """ The actual code for the lambda function."""

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

    description: Optional[str] = None
    """ Textual description of what this function does. """


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
        network: Network,
        override_name: Optional[str] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:Lambda", name, None, opts)

        # Our previous usage of CDK was such that we automatically
        # created a separate security group for each of our lambdas:
        #
        # https://docs.aws.amazon.com/cdk/api/latest/docs/@aws-cdk_aws-lambda.Function.html#securitygroups
        self.security_group = aws.ec2.SecurityGroup(
            f"{name}-security-group",
            vpc_id=network.vpc.id,
            opts=pulumi.ResourceOptions(parent=self),
        )

        lambda_name = f"{DEPLOYMENT_NAME}-{name}"
        self.function = aws.lambda_.Function(
            f"{name}-lambda",
            name=override_name or name,
            description=args.description,
            runtime=args.runtime,
            package_type=args.package_type,
            handler=args.handler,
            code=args.code,
            environment=aws.lambda_.FunctionEnvironmentArgs(variables=args.env),
            memory_size=args.memory_size,
            timeout=args.timeout,
            role=args.execution_role.arn,
            vpc_config=aws.lambda_.FunctionVpcConfigArgs(
                # See https://docs.aws.amazon.com/lambda/latest/dg/configuration-vpc.html
                security_group_ids=[self.security_group.id],
                subnet_ids=[net.id for net in network.private_subnets],
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

        if not LOCAL_GRAPL:
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
                opts=pulumi.ResourceOptions(parent=self),
            )

        # Lambda function output is automatically sent to log
        # groups named like this; we denote this one explicitly so we can
        # specify retention rules.
        self.log_group = aws.cloudwatch.LogGroup(
            f"{name}-log-group",
            # Don't change - or rather, if you decide to,
            # follow these instructions:
            # https://www.pulumi.com/docs/reference/pkg/aws/lambda/function/#cloudwatch-logging-and-permissions
            name=f"/aws/lambda/{lambda_name}",
            retention_in_days=SERVICE_LOG_RETENTION_DAYS,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})
