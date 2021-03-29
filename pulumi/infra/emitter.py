import json
from typing import Optional

import pulumi_aws as aws
from infra.bucket import Bucket, bucket_physical_name
from infra.config import AWS_ACCOUNT_ID, DEPLOYMENT_NAME, import_aware_opts

import pulumi


class EventEmitter(pulumi.ComponentResource):
    """
    Buckets that send events to SNS topics.
    """

    def __init__(
        self, event_name: str, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:

        super().__init__("grapl:EventEmitter", event_name, None, opts)

        logical_bucket_name = f"{event_name}-bucket"
        self.bucket = Bucket(logical_bucket_name, sse=True, parent=self)

        region = aws.get_region().name
        physical_topic_name = f"{DEPLOYMENT_NAME}-{event_name}-topic"
        topic_lookup_arn = (
            f"arn:aws:sns:{region}:{AWS_ACCOUNT_ID}:{physical_topic_name}"
        )
        self.topic = aws.sns.Topic(
            f"{event_name}-topic",
            name=physical_topic_name,
            opts=import_aware_opts(topic_lookup_arn, parent=self),
        )

        # This is a resource-based policy to allow our bucket to
        # publish to our topic, which in turn allows us to set up the
        # bucket notification below.
        self.topic_policy_attachment = aws.sns.TopicPolicy(
            f"{event_name}-bucket-publishes-to-topic",
            arn=self.topic.arn,
            policy=pulumi.Output.all(self.topic.arn, self.bucket.arn).apply(
                lambda topic_and_bucket: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Sid": "0",
                                "Effect": "Allow",
                                "Principal": {
                                    "Service": "s3.amazonaws.com",
                                },
                                "Action": "sns:Publish",
                                "Resource": topic_and_bucket[0],
                                "Condition": {
                                    "ArnLike": {"aws:SourceArn": topic_and_bucket[1]}
                                },
                            }
                        ],
                    }
                )
            ),
            opts=import_aware_opts(topic_lookup_arn, parent=self),
        )

        self.bucket_notification = aws.s3.BucketNotification(
            f"{logical_bucket_name}-notifies-topic",
            bucket=self.bucket.id,
            topics=[
                aws.s3.BucketNotificationTopicArgs(
                    topic_arn=self.topic.arn,
                    events=["s3:ObjectCreated:*"],
                )
            ],
            # Ideally, I'd like to use `self.bucket.id` for this, but
            # that isn't going to be available from the Pulumi
            # resource at planning time, which is when we'd need this
            # string.  However, we're only going to need this while we
            # straddle CDK and Pulumi; it'll go away once we're
            # totally migrated.
            opts=import_aware_opts(
                bucket_physical_name(logical_bucket_name), parent=self
            ),
        )

        self.register_outputs({})
