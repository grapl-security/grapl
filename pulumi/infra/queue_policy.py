"""
Consolidates various SQS queue policies in one place.
"""

import json

import pulumi_aws as aws

import pulumi


def consumption_policy(queue: aws.sqs.Queue, role: aws.iam.Role) -> None:
    """
    Adds an inline policy to `role` for consuming messages from `queue`.

    The resulting `RolePolicy` resource is a child of the role.
    """
    aws.iam.RolePolicy(
        f"{role._name}-consumes-from-{queue._name}",
        role=role.name,
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
        opts=pulumi.ResourceOptions(parent=role),
    )


def allow_send_from_topic(queue: aws.sqs.Queue, topic: aws.sns.Topic) -> None:
    """
    Set a policy on Queue
    that allows SendMessages from Topic
    """
    policy = pulumi.Output.all(queue_arn=queue.arn, topic_arn=topic.arn).apply(
        lambda input: json.dumps(
            {
                "Version": "2012-10-17",
                "Statement": [
                    {
                        "Effect": "Allow",
                        "Principal": {"Service": "sns.amazonaws.com"},
                        "Action": [
                            "sqs:SendMessage",
                        ],
                        "Resource": input["queue_arn"],
                        "Condition": {
                            "ArnEquals": {"aws:SourceArn": input["topic_arn"]}
                        },
                    }
                ],
            }
        )
    )
    aws.sqs.QueuePolicy(
        f"allow-send-from-{topic._name}",
        queue_url=queue.id,
        policy=policy,
        opts=pulumi.ResourceOptions(parent=queue),
    )
