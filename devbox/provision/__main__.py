# Pants would resolve this import correctly for us, but since we run this
# directly through Pulumi sans-Pants we need this grossness
import sys

sys.path.insert(0, "..")

import pulumi_aws as aws
from provision.infra.ami import get_ami
from provision.infra.iam_instance_profile import IamInstanceProfile
from provision.infra.security_group import SecurityGroup

import pulumi


def main() -> None:
    config = pulumi.Config()
    instance_type = config.require("instance-type")

    security_group = SecurityGroup(name="security-group")

    ami = get_ami()

    instance_profile = IamInstanceProfile("instance-profile")

    # Notably, this public/private key is not the same one you use with Github
    # but a bespoke one generated just for Devbox usage.
    public_key_name = "devbox-public-key"
    public_key = aws.ec2.KeyPair(
        public_key_name,
        key_name=public_key_name,
        public_key=config.require("public-key"),
    )

    instance_name = "devbox-instance"
    instance = aws.ec2.Instance(
        instance_name,
        ami=ami.id,
        iam_instance_profile=instance_profile.instance_profile.name,
        instance_type=instance_type,
        key_name=public_key.id,
        vpc_security_group_ids=[security_group.security_group.id],
        tags={
            "Name": instance_name,
        },
    )

    pulumi.export("devbox-instance-id", instance.id)


if __name__ == "__main__":
    main()
