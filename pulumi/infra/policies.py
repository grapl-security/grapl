import json

import pulumi_aws as aws

import pulumi


def _attach_AmazonSSMManagedInstanceCore(
    role: aws.iam.Role, opts: pulumi.ResourceOptions
) -> aws.iam.RolePolicyAttachment:
    """
    https://github.com/grapl-security/issue-tracker/issues/106
    Moving away from managed server policies.
    """
    policy_name = "demanaged_AmazonSSMManagedInstanceCore"
    policy = aws.iam.Policy(
        policy_name,
        policy=json.dumps(
            {
                "Version": "2012-10-17",
                "Statement": [
                    {
                        "Effect": "Allow",
                        "Action": [
                            "ssm:DescribeAssociation",
                            "ssm:GetDeployablePatchSnapshotForInstance",
                            "ssm:GetDocument",
                            "ssm:DescribeDocument",
                            "ssm:GetManifest",
                            "ssm:GetParameter",
                            "ssm:GetParameters",
                            "ssm:ListAssociations",
                            "ssm:ListInstanceAssociations",
                            "ssm:PutInventory",
                            "ssm:PutComplianceItems",
                            "ssm:PutConfigurePackageResult",
                            "ssm:UpdateAssociationStatus",
                            "ssm:UpdateInstanceAssociationStatus",
                            "ssm:UpdateInstanceInformation",
                        ],
                        "Resource": "*",
                    },
                    {
                        "Effect": "Allow",
                        "Action": [
                            "ssmmessages:CreateControlChannel",
                            "ssmmessages:CreateDataChannel",
                            "ssmmessages:OpenControlChannel",
                            "ssmmessages:OpenDataChannel",
                        ],
                        "Resource": "*",
                    },
                    {
                        "Effect": "Allow",
                        "Action": [
                            "ec2messages:AcknowledgeMessage",
                            "ec2messages:DeleteMessage",
                            "ec2messages:FailMessage",
                            "ec2messages:GetEndpoint",
                            "ec2messages:GetMessages",
                            "ec2messages:SendReply",
                        ],
                        "Resource": "*",
                    },
                ],
            }
        ),
        opts=opts,
    )
    return aws.iam.RolePolicyAttachment(
        f"attach_{policy_name}",
        role=role.name,
        policy_arn=policy.arn,
        opts=opts,
    )


def _attach_CloudWatchAgentServerPolicy(
    role: aws.iam.Role, opts: pulumi.ResourceOptions
) -> aws.iam.RolePolicyAttachment:
    """
    https://github.com/grapl-security/issue-tracker/issues/106
    Moving away from managed server policies.
    """
    policy_name = "demanaged_CloudWatchAgentServerPolicy"
    policy = aws.iam.Policy(
        policy_name,
        policy=json.dumps(
            {
                "Version": "2012-10-17",
                "Statement": [
                    {
                        "Effect": "Allow",
                        "Action": [
                            "cloudwatch:PutMetricData",
                            "ec2:DescribeVolumes",
                            "ec2:DescribeTags",
                            "logs:PutLogEvents",
                            "logs:DescribeLogStreams",
                            "logs:DescribeLogGroups",
                            "logs:CreateLogStream",
                            "logs:CreateLogGroup",
                        ],
                        "Resource": "*",
                    },
                    {
                        "Effect": "Allow",
                        "Action": ["ssm:GetParameter"],
                        "Resource": "arn:aws:ssm:*:*:parameter/AmazonCloudWatch-*",
                    },
                ],
            }
        ),
        opts=opts,
    )
    return aws.iam.RolePolicyAttachment(
        f"attach_{policy_name}",
        role=role.name,
        policy_arn=policy.arn,
        opts=opts,
    )


def _attach_policy_to_ship_logs_to_cloudwatch(
    role: aws.iam.Role, log_group: aws.cloudwatch.LogGroup, opts: pulumi.ResourceOptions
) -> aws.iam.RolePolicyAttachment:
    policy_name = "policy_to_ship_logs_to_cloudwatch"
    policy = log_group.arn.apply(
        lambda arn: aws.iam.Policy(
            policy_name,
            policy=json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Action": [
                                "logs:CreateLogGroup",
                                "logs:CreateLogStream",
                                "logs:PutLogEvents",
                                "logs:DescribeLogStreams",
                            ],
                            "Resource": f"{arn}:*",
                        }
                    ],
                }
            ),
            opts=opts,
        ),
    )
    return aws.iam.RolePolicyAttachment(
        f"attach_{policy_name}",
        role=role.name,
        policy_arn=policy.arn,
        opts=opts,
    )
