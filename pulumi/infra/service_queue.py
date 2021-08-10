import json
from typing import NamedTuple, Optional

import pulumi_aws as aws
from infra import queue_policy
from infra.emitter import EventEmitter

import pulumi


class ServiceQueueNames(NamedTuple):
    service_name: str
    queue: str
    retry_queue: str
    dead_letter_queue: str


class ServiceConfiguration(NamedTuple):
    """Encapsulates the information needed to configure a service to interact with a `ServiceQueue`.

    In particular, services will have one queue from which they will
    pull messages (the "main queue"), and one queue to which they will
    write messages that could not be processed (the "dead-letter
    queue"). Depending on whether the service is a "default" service
    or a "retry" service, the specific identities of these queues will
    be different.

    """

    main_queue: aws.sqs.Queue
    dead_letter_queue: aws.sqs.Queue

    @property
    def main_url(self) -> pulumi.Output[str]:
        """ The URL of the main queue."""
        return self.main_queue.id

    @property
    def dead_letter_url(self) -> pulumi.Output[str]:
        """ The URL of the dead-letter queue."""
        return self.dead_letter_queue.id

    def grant_queue_permissions_to(self, role: aws.iam.Role) -> None:
        """Adds an inline policy to `role` for consuming messages from
        `main_queue` and writing messages to `dead_letter_queue`.

        The resulting `RolePolicy` resource is a child of the role.

        """
        aws.iam.RolePolicy(
            f"{role._name}-reads-{self.main_queue._name}-writes-{self.dead_letter_queue._name}",
            role=role.name,
            policy=pulumi.Output.all(
                main_arn=self.main_queue.arn, dead_letter_arn=self.dead_letter_queue.arn
            ).apply(
                lambda inputs: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Sid": "ReadFromMainQueue",
                                "Effect": "Allow",
                                "Action": [
                                    "sqs:ChangeMessageVisibility",
                                    "sqs:DeleteMessage",
                                    "sqs:GetQueueAttributes",
                                    "sqs:GetQueueUrl",
                                    "sqs:ReceiveMessage",
                                ],
                                "Resource": inputs["main_arn"],
                            },
                            {
                                "Sid": "WriteToDeadLetterQueue",
                                "Effect": "Allow",
                                "Action": [
                                    "sqs:SendMessage",
                                    "sqs:GetQueueAttributes",
                                    "sqs:GetQueueUrl",
                                ],
                                "Resource": inputs["dead_letter_arn"],
                            },
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=role),
        )


class ServiceQueue(pulumi.ComponentResource):
    """
    Each service currently deals with three queues. The main queue
    falls back to a "retry queue", which itself falls back to a "dead
    letter" queue.
    """

    def __init__(
        self, name: str, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:
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

        dead_letter_name = f"{name}-dead-letter-queue"
        self.dead_letter_queue = aws.sqs.Queue(
            dead_letter_name,
            message_retention_seconds=message_retention_seconds,
            visibility_timeout_seconds=30,
            opts=pulumi.ResourceOptions(
                parent=self,
            ),
        )
        pulumi.export(dead_letter_name, self.dead_letter_queue_url)

        retry_name = f"{name}-retry-queue"
        self.retry_queue = aws.sqs.Queue(
            retry_name,
            message_retention_seconds=message_retention_seconds,
            visibility_timeout_seconds=360,
            redrive_policy=self.dead_letter_queue.arn.apply(redrive_policy),
            opts=pulumi.ResourceOptions(
                parent=self,
            ),
        )
        pulumi.export(retry_name, self.retry_queue_url)

        queue_name = f"{name}-queue"
        self.queue = aws.sqs.Queue(
            queue_name,
            message_retention_seconds=message_retention_seconds,
            visibility_timeout_seconds=180,
            redrive_policy=self.retry_queue.arn.apply(redrive_policy),
            opts=pulumi.ResourceOptions(
                parent=self,
            ),
        )
        pulumi.export(queue_name, self.main_queue_url)

        self.register_outputs({})

    # Yes, the `id` property of an SQS queue is actually a URL.
    #
    # The URL generated by Pulumi is currently of the form:
    #     https://sqs.{AWS_REGION}.amazonaws.com/{ACCOUNT_ID}/{QUEUE_NAME}
    #
    # FYI: A URL in the form of:
    #     https://queue.amazonaws.com/{ACCOUNT_ID}/{QUEUE_NAME}
    # would also be valid, but this is not what Pulumi generates.
    #
    # We expose these URLs as properties on ServiceQueue to:
    #
    # a) encapsulate things a bit better
    # b) only have one place that needs to know that "id == URL"
    # c) have one place to modify if this behavior ever changes in Pulumi

    @property
    def main_queue_url(self) -> pulumi.Output[str]:
        return self.queue.id

    @property
    def retry_queue_url(self) -> pulumi.Output[str]:
        return self.retry_queue.id

    @property
    def dead_letter_queue_url(self) -> pulumi.Output[str]:
        return self.dead_letter_queue.id

    @property
    def queue_names(self) -> pulumi.Output[ServiceQueueNames]:
        """
        Helps de-complicate creating dashboards off this service.
        """
        return pulumi.Output.all(
            self._name,
            self.queue.name,
            self.retry_queue.name,
            self.dead_letter_queue.name,
        ).apply(lambda args: ServiceQueueNames(*args))

    def subscribe_to_emitter(self, emitter: EventEmitter) -> None:
        """
        Enable this queue to be fed by events from `emitter`.
        """
        aws.sns.TopicSubscription(
            f"{self.queue._name}-subscribes-to-{emitter.topic._name}",
            protocol="sqs",
            endpoint=self.queue.arn,
            topic=emitter.topic.arn,
            raw_message_delivery=True,
            opts=pulumi.ResourceOptions(parent=emitter.topic),
        )

        queue_policy.allow_send_from_topic(self.queue, emitter.topic)

    def default_service_configuration(self) -> ServiceConfiguration:
        """
        Information needed to configure a "default" service to interact with this `ServiceQueue`.
        """
        return ServiceConfiguration(self.queue, self.retry_queue)

    def retry_service_configuration(self) -> ServiceConfiguration:
        """Information needed to configure a "retry" service to interact with
        this `ServiceQueue`.

        """
        return ServiceConfiguration(self.retry_queue, self.dead_letter_queue)
