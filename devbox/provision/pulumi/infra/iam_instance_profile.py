import json
import pulumi
import pulumi_aws as aws
from infra.demanaged_policies import attach_policy, get_ssm_policy

class IamInstanceProfile(pulumi.ComponentResource):
    def __init__(self, name: str, opts: pulumi.ResourceOptions=None) -> None:
        super().__init__("devbox:IamInstanceProfile", name=name, props=None, opts=opts)
        devbox_role = aws.iam.Role(
            "devbox-instance-role",
            description="Devbox Instance role",
            assume_role_policy=json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Action": "sts:AssumeRole",
                            "Effect": "Allow",
                            "Principal": {
                                "Service": ["ec2.amazonaws.com"],
                            },
                        }
                    ],
                }
            ),
            opts=pulumi.ResourceOptions(parent=self)
        )
        ssm_policy = get_ssm_policy(
            opts=pulumi.ResourceOptions(parent=self)
        )
        attach_policy(ssm_policy, devbox_role)

        self.instance_profile = aws.iam.InstanceProfile(
            "devbox-instance-profile",
            role=devbox_role.name,
            opts=pulumi.ResourceOptions(parent=self),
        )