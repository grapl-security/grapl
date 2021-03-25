import json

import pulumi_aws as aws
from infra.config import (
    AWS_ACCOUNT_ID,
    DEPLOYMENT_NAME,
    GLOBAL_LAMBDA_ZIP_TAG,
    import_aware_opts,
    mg_alphas,
)
from infra.emitter import EventEmitter
from infra.lambda_ import code_path_for
from infra.metric_forwarder import MetricForwarder
from infra.service import Service

import pulumi


class EngagementCreator(Service):
    def __init__(
        self, source_emitter: EventEmitter, forwarder: MetricForwarder
    ) -> None:

        name = "engagement-creator"
        super().__init__(
            name,
            forwarder=forwarder,
            lambda_description=GLOBAL_LAMBDA_ZIP_TAG,
            lambda_handler_fn="lambdex_handler.handler",
            lambda_code_path=code_path_for("engagement-creator"),
            env={
                "GRAPL_LOG_LEVEL": "INFO",
                "MG_ALPHAS": mg_alphas(),
            },
        )

        self.bucket_policy_attachment = aws.iam.RolePolicy(
            f"{name}-reads-from-emitter-bucket",
            name=f"{DEPLOYMENT_NAME}-{name}-reads-from-emitter-bucket",
            role=self.role.name,
            policy=source_emitter.bucket.arn.apply(
                lambda bucket_arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Sid": "0",
                                "Effect": "Allow",
                                "Action": "s3:GetObject",
                                "Resource": f"{bucket_arn}/*",
                            }
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

        # NOTE: The main queue from the service's ServiceQueue is being
        # wired up to the topic from the event emitter.... but this
        # only seems to happen for the Engagement Creator service, and
        # not generally. We may want to revisit how things are
        # abstracted to reflect this a bit more cleanly.
        self.subscription = aws.sns.TopicSubscription(
            f"{name}-subscribes-to-emitter-topic",  # TODO more descriptive
            protocol="sqs",
            endpoint=self.queue.queue.arn,
            topic=source_emitter.topic.arn,
            raw_message_delivery=True,
            opts=pulumi.ResourceOptions(parent=self),
        )

        region = aws.get_region().name
        physical_topic_name = f"{DEPLOYMENT_NAME}-engagements-created-topic"
        topic_lookup_arn = (
            f"arn:aws:sns:{region}:{AWS_ACCOUNT_ID}:{physical_topic_name}"
        )
        self.created_topic = aws.sns.Topic(
            "engagements-created-topic",
            name=physical_topic_name,
            opts=import_aware_opts(topic_lookup_arn, parent=self),
        )

        publish_to_topic_policy = self.created_topic.arn.apply(
            lambda topic_arn: json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            # TODO: Do we need CreateTopic? In any
                            # event, this is what was in our CDK code
                            "Action": ["sns:CreateTopic", "sns:Publish"],
                            "Resource": topic_arn,
                        }
                    ],
                }
            )
        )

        self.topic_policy_attachment = aws.iam.RolePolicy(
            f"{name}-publishes-to-topic",
            name=f"{DEPLOYMENT_NAME}-{name}-publishes-to-topic",
            role=self.role.name,
            policy=publish_to_topic_policy,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})
