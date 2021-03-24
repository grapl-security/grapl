import json
from typing import Optional

import pulumi_aws as aws
from infra import util
from infra.util import AWS_ACCOUNT_ID, DEPLOYMENT_NAME

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

        # Queues have to be imported by URL, which includes the
        # account ID
        queue_import_prefix = f"https://queue.amazonaws.com/{AWS_ACCOUNT_ID}"

        logical_dead_letter_name = f"{name}-dead-letter-queue"
        physical_dead_letter_name = f"{DEPLOYMENT_NAME}-{logical_dead_letter_name}"
        self.dead_letter_queue = aws.sqs.Queue(
            logical_dead_letter_name,
            name=physical_dead_letter_name,
            message_retention_seconds=message_retention_seconds,
            visibility_timeout_seconds=30,
            opts=util.import_aware_opts(
                f"{queue_import_prefix}/{physical_dead_letter_name}",
                parent=self,
                delete_before_replace=True,
            ),
        )

        logical_retry_name = f"{name}-retry-queue"
        physical_retry_name = f"{DEPLOYMENT_NAME}-{logical_retry_name}"
        self.retry_queue = aws.sqs.Queue(
            logical_retry_name,
            name=physical_retry_name,
            message_retention_seconds=message_retention_seconds,
            visibility_timeout_seconds=360,
            redrive_policy=self.dead_letter_queue.arn.apply(redrive_policy),
            opts=util.import_aware_opts(
                f"{queue_import_prefix}/{physical_retry_name}",
                parent=self,
                delete_before_replace=True,
            ),
        )

        logical_queue_name = f"{name}-queue"
        physical_queue_name = f"{DEPLOYMENT_NAME}-{logical_queue_name}"
        self.queue = aws.sqs.Queue(
            logical_queue_name,
            name=physical_queue_name,
            message_retention_seconds=message_retention_seconds,
            visibility_timeout_seconds=180,
            redrive_policy=self.retry_queue.arn.apply(redrive_policy),
            opts=util.import_aware_opts(
                f"{queue_import_prefix}/{physical_queue_name}",
                parent=self,
                delete_before_replace=True,
            ),
        )

        # TODO Purge queues? This was in the old code; not sure if needed

        self.register_outputs({})
