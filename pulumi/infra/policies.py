import json

import pulumi_aws as aws
from typing_extensions import Final

import pulumi

SSM_POLICY: Final[aws.iam.Policy] = aws.iam.Policy(
    "demanaged-AmazonSSMManagedInstanceCore",
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
)

CLOUDWATCH_AGENT_SERVER_POLICY: Final[aws.iam.Policy] = aws.iam.Policy(
    "demanaged-CloudWatchAgentServerPolicy",
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
)


def attach_policy(
    role: aws.iam.Role, policy: aws.iam.Policy
) -> aws.iam.RolePolicyAttachment:
    """
    https://github.com/grapl-security/issue-tracker/issues/106
    Moving away from managed server policies.
    """

    return aws.iam.RolePolicyAttachment(
        f"attach-{policy._name}-to-{role._name}",
        role=role.name,
        policy_arn=policy.arn,
        opts=pulumi.ResourceOptions(parent=policy),
    )


def _attach_policy_to_ship_logs_to_cloudwatch(
    role: aws.iam.Role, log_group: aws.cloudwatch.LogGroup, opts: pulumi.ResourceOptions
) -> aws.iam.RolePolicyAttachment:
    # This seems like it's a strict subset of CLOUDWATCH_AGENT_SERVER_POLICY
    # and there's a good chance it can be removed.
    policy = aws.iam.Policy(
        "policy-to-ship-logs-to-cloudwatch",
        policy=log_group.arn.apply(
            lambda arn: json.dumps(
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
            )
        ),
        opts=opts,
    )
    return attach_policy(role=role, policy=policy)
