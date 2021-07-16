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

EC2_DESCRIBE_INSTANCES_POLICY: Final[aws.iam.Policy] = aws.iam.Policy(
    'ec2-DescribeInstances',
    policy=json.dumps(
        {
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": [
                        "ec2:DescribeTags",
                        "ec2:DescribeInstances",
                        "autoscaling:DescribeAutoScalingGroups"
                    ],
                    "Resource": "*"
                }
            ]
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

ECR_TOKEN_POLICY: Final[aws.iam.Policy] = aws.iam.Policy(
    "ECR-authorization-token-policy",
    description="Get ECR Authorization Tokens",
    policy=json.dumps(
        {
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": "ecr:GetAuthorizationToken",
                    "Resource": "*",
                }
            ],
        }
    ),
)


def attach_policy(
    policy: aws.iam.Policy, role: aws.iam.Role
) -> aws.iam.RolePolicyAttachment:
    """Attaches the `policy` to the `role`.

    The resulting `RolePolicyAttachment` is created as a child
    resource of the policy. Naming of the resource is also handled.

    Prefer this over the direct creation of a `RolePolicyAttachment`
    whenever possible to promote consistency across our
    infrastructure.

    """
    return aws.iam.RolePolicyAttachment(
        f"attach-{policy._name}-to-{role._name}",
        role=role.name,
        policy_arn=policy.arn,
        opts=pulumi.ResourceOptions(parent=policy),
    )
