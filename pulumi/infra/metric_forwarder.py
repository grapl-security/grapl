import json
from typing import Optional

import pulumi_aws as aws
from infra.config import GLOBAL_LAMBDA_ZIP_TAG

import pulumi


# TODO: will need VPC here
class MetricForwarder(pulumi.ComponentResource):
    def __init__(self, opts: Optional[pulumi.ResourceOptions] = None) -> None:
        name = "metric-forwarder"
        super().__init__("grapl:MetricForwarder", name, None, opts)

        # Importing here avoids circular import hell between
        # MetricForwarder and Lambda
        from infra.lambda_ import Lambda, LambdaArgs, LambdaExecutionRole, code_path_for

        self.role = LambdaExecutionRole(
            name,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.function = Lambda(
            name,
            args=LambdaArgs(
                description=GLOBAL_LAMBDA_ZIP_TAG,
                execution_role=self.role,
                runtime=aws.lambda_.Runtime.CUSTOM_AL2,
                handler="metric-forwarder",
                code_path=code_path_for("metric-forwarder"),
                package_type="Zip",
                env={"GRAPL_LOG_LEVEL": "INFO", "RUST_BACKTRACE": "1"},
                memory_size=128,
                timeout=45,
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

        # This allows us to write our metrics to cloudwatch
        self.cloudwatch_policy = aws.iam.RolePolicy(
            f"{name}-writes-to-cloudwatch",
            role=self.role.name,
            policy=json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Action": "cloudwatch:PutMetricData",
                            "Resource": "*",
                        }
                    ],
                }
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})
