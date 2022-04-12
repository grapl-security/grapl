# Pants would resolve this import correctly for us, but since we run this
# directly through Pulumi sans-Pants we need this grossness
import sys

sys.path.insert(0, "..")

from typing import Mapping

import pulumi_aws as aws
from provision.infra.ami import get_ami
from provision.infra.iam_instance_profile import IamInstanceProfile
from provision.infra.security_group import SecurityGroup
from typing_extensions import Final

import pulumi

class DevboxProvisionConfig:
    def __init__(self, pulumi_config: pulumi.Config) -> None:
        self.instance_type = pulumi_config.require("instance-type")
        self.public_key = pulumi_config.require("public-key")
        self.instance_volume_size_gb = pulumi_config.require_int("instance-volume-size-gb")


def main() -> None:
    config = DevboxProvisionConfig(pulumi.Config())

    tags: Final[Mapping[str, str]] = {
        "pulumi:project": pulumi.get_project(),
        "pulumi:stack": pulumi.get_stack(),
    }

    security_group = SecurityGroup(name="security-group", tags=tags)
    instance_profile = IamInstanceProfile("instance-profile", tags=tags)
    ami = get_ami()

    # Notably, this public/private key is not the same one you use with Github
    # but a bespoke one generated just for Devbox usage.
    # It is created in `provision.sh`.
    public_key_name = "devbox-public-key"
    public_key = aws.ec2.KeyPair(
        public_key_name,
        key_name=public_key_name,
        public_key=config.public_key,
        tags=tags,
    )

    instance_name = "devbox-instance"
    instance = aws.ec2.Instance(
        instance_name,
        ami=ami.id,
        iam_instance_profile=instance_profile.instance_profile.name,
        instance_type=config.instance_type,
        key_name=public_key.id,
        root_block_device=aws.ec2.InstanceRootBlockDeviceArgs(
            volume_size=config.instance_volume_size_gb,
        ),
        vpc_security_group_ids=[security_group.security_group.id],
        tags={
            "Name": instance_name,
            **tags,
        },
    )

    pulumi.export("devbox-instance-id", instance.id)


if __name__ == "__main__":
    main()
