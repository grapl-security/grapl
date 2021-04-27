import json
from typing import Optional

import pulumi_aws as aws
from infra.lambda_ import Lambda, LambdaArgs
from infra.metric_forwarder import MetricForwarder
from infra.network import Network

import pulumi


class QueueDrivenLambda(pulumi.ComponentResource):
    """ A lambda function that is triggered by an SQS queue. """

    def __init__(
        self,
        name: str,
        queue: aws.sqs.Queue,
        args: LambdaArgs,
        network: Network,
        forwarder: Optional[MetricForwarder] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        super().__init__("grapl:QueueDrivenLambda", name, None, opts)

        self.function = Lambda(
            name,
            args=args,
            network=network,
            forwarder=forwarder,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.policy = aws.iam.RolePolicy(
            f"{name}-consumes-from-queue",
            role=args.execution_role.name,
            policy=queue.arn.apply(
                lambda arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Effect": "Allow",
                                "Action": [
                                    "sqs:ChangeMessageVisibility",
                                    "sqs:DeleteMessage",
                                    "sqs:GetQueueAttributes",
                                    "sqs:GetQueueUrl",
                                    "sqs:ReceiveMessage",
                                ],
                                "Resource": arn,
                            }
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.event_source_mapping = aws.lambda_.EventSourceMapping(
            f"queue-triggers-{name}",
            event_source_arn=queue.arn,
            function_name=self.function.function.arn,
            batch_size=10,  # Default value for SQS queues
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})
