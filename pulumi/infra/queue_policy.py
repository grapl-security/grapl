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


def send_policy(queue: aws.sqs.Queue, role: aws.iam.Role) -> None:
    """
    Adds an inline policy to `role` for sending messages into `queue`.

    The resulting `RolePolicy` resource is a child of the role.
    """
    aws.iam.RolePolicy(
        f"{role._name}-writes-to-{queue._name}",
        role=role.name,
        policy=queue.arn.apply(
            lambda arn: json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Action": [
                                "sqs:SendMessage",
                                "sqs:GetQueueAttributes",
                                "sqs:GetQueueUrl",
                            ],
                            "Resource": arn,
                        }
                    ],
                }
            )
        ),
        opts=pulumi.ResourceOptions(parent=role),
    )
