import pulumi_aws as aws
from infra.ami import get_ami
from infra.iam_instance_profile import IamInstanceProfile
from infra.security_group import SecurityGroup

import pulumi


def main() -> None:
    config = pulumi.Config()
    instance_type = config.require("instance-type")

    security_group = SecurityGroup(name="security-group")

    ami = get_ami()

    instance_profile = IamInstanceProfile("instance-profile")

    instance_name = "devbox-instance"
    instance = aws.ec2.Instance(
        instance_name,
        instance_type=instance_type,
        iam_instance_profile=instance_profile.instance_profile.name,
        ami=ami.id,
        vpc_security_group_ids=[security_group.security_group.id],
        tags={
            "Name": instance_name,
        },
    )

    pulumi.export("devbox-instance-id", instance.id)


if __name__ == "__main__":
    main()
