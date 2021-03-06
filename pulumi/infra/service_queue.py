import json
from typing import Optional

import pulumi_aws as aws

import pulumi


class ServiceQueue(pulumi.ComponentResource):
    """
    Each service currently deals with three queues. The main queue
    falls back to a "retry queue", which itself falls back to a "dead
    letter" queue.
    """

    def __init__(
        self, name: str, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:
        # TODO: grapl_infra? grapl.infra? grapl:service:servicequeue?
        super().__init__("grapl:ServiceQueue", name, None, opts)

        message_retention_seconds = 60 * 60 * 24 * 4  # 4 days

        # `arn` is the ARN of a queue. This is a function because of
        # the need to use Output.apply on the ARN.
        def redrive_policy(arn: pulumi.Output) -> str:
            return json.dumps(
                {
                    "deadLetterTargetArn": arn,
                    "maxReceiveCount": 3,
                }
            )

        # TODO: delete_before_replace is only needed if we're
        # overriding the name of the queues

        prefix = pulumi.get_stack()

        dead_letter_name = f"{prefix}-{name}-dead_letter-queue"
        self.dead_letter_queue = aws.sqs.Queue(
            dead_letter_name,
            name=dead_letter_name,
            message_retention_seconds=message_retention_seconds,
            opts=pulumi.ResourceOptions(parent=self, delete_before_replace=True),
            visibility_timeout_seconds=30,
        )

        retry_name = f"{prefix}-{name}-retry-queue"
        self.retry_queue = aws.sqs.Queue(
            retry_name,
            name=retry_name,
            message_retention_seconds=message_retention_seconds,
            visibility_timeout_seconds=360,
            redrive_policy=self.dead_letter_queue.arn.apply(redrive_policy),
            opts=pulumi.ResourceOptions(parent=self, delete_before_replace=True),
        )

        queue_name = f"{prefix}-{name}-queue"
        self.queue = aws.sqs.Queue(
            queue_name,
            name=queue_name,
            message_retention_seconds=message_retention_seconds,
            visibility_timeout_seconds=180,
            redrive_policy=self.retry_queue.arn.apply(redrive_policy),
            opts=pulumi.ResourceOptions(parent=self, delete_before_replace=True),
        )

        # TODO Purge queues? This was in the old code; not sure if needed

        self.register_outputs({})
