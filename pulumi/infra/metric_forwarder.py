import json
from typing import Optional

import pulumi_aws as aws
from infra.config import configurable_envvars
from infra.lambda_ import Lambda, LambdaArgs, LambdaExecutionRole, code_path_for
from infra.network import Network

import pulumi


class MetricForwarder(pulumi.ComponentResource):
    def __init__(
        self, network: Network, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:
        name = "metric-forwarder"
        super().__init__("grapl:MetricForwarder", name, None, opts)

        self.role = LambdaExecutionRole(
            name,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.function = Lambda(
            name,
            args=LambdaArgs(
                execution_role=self.role,
                runtime=aws.lambda_.Runtime.CUSTOM_AL2,
                handler="metric-forwarder",
                code_path=code_path_for("metric-forwarder"),
                package_type="Zip",
                env={
                    **configurable_envvars(
                        "metric-forwarder", ["RUST_LOG", "RUST_BACKTRACE"]
                    ),
                },
                memory_size=128,
                timeout=45,
            ),
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        # This allows us to write our metrics to cloudwatch
        aws.iam.RolePolicy(
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
            opts=pulumi.ResourceOptions(parent=self.role),
        )

        self.register_outputs({})

    def subscribe_to_log_group(
        self, name: str, log_group: aws.cloudwatch.LogGroup
    ) -> None:
        """
        Allows the metric forwarder to process `MONITORING` logs from the given log group.

        `name` is used to create Pulumi resource names; it should reflect the thing feeding into the log group.
        """

        # Allow the metric forwarder to be invoked with by this
        # lambda function's log group.
        permission = aws.lambda_.Permission(
            f"{name}-log-group-invokes-metric-forwarder",
            principal=f"logs.amazonaws.com",
            action="lambda:InvokeFunction",
            function=self.function.function.arn,
            source_arn=log_group.arn.apply(lambda arn: f"{arn}:*"),
            opts=pulumi.ResourceOptions(parent=self),
        )

        # Then, with that permission granted, configure the metric
        # forwarder to receive all 'MONITORING' log messages from
        # the lambda function.
        aws.cloudwatch.LogSubscriptionFilter(
            f"metric-forwarder-subscribes-to-{name}-monitoring-logs",
            log_group=log_group.name,
            filter_pattern="MONITORING",
            destination_arn=self.function.function.arn,
            opts=pulumi.ResourceOptions(depends_on=[permission], parent=self),
        )
