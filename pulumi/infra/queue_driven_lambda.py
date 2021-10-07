from typing import Optional

import pulumi_aws as aws
from infra import queue_policy
from infra.lambda_ import Lambda, LambdaArgs
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
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        super().__init__("grapl:QueueDrivenLambda", name, None, opts)

        self.function = Lambda(
            name,
            args=args,
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        queue_policy.consumption_policy(queue, args.execution_role)

        self.event_source_mapping = aws.lambda_.EventSourceMapping(
            f"queue-triggers-{name}",
            event_source_arn=queue.arn,
            function_name=self.function.function.arn,
            batch_size=10,  # Default value for SQS queues
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})
