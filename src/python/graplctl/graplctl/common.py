from __future__ import annotations

import dataclasses
import time

from typing import List

import mypy_boto3_ec2.service_resource as ec2_resources
from mypy_boto3_ssm.client import SSMClient


@dataclasses.dataclass
class Tag:
    key: str
    value: str

    def into_boto_tag_specification(self) -> Dict[str, str]:
        return {"Key": self.key, "Value": self.value}

    @classmethod
    def from_boto_tag_specification(cls, tag_specification: Dict[str, str]) -> Tag:
        return cls(key=tag_specification["Key"], value=tag_specification["Value"])


@dataclasses.dataclass
class Ec2Instance:
    instance_id: str
    private_ip_address: str
    private_dns_name: str
    tags: List[Tag]

    @classmethod
    def from_boto_instance(cls, instance: ec2_resources.Instance) -> Ec2Instance:
        return cls(
            instance_id=instance.instance_id,
            private_ip_address=instance.private_ip_address,
            private_dns_name=instance.private_dns_name,
            tags=[Tag.from_boto_tag_specification(tag) for tag in instance.tags],
        )


def get_command_result(ssm: SSMClient, command_id: str, instance_id: str) -> str:
    """Poll until the command result is available for the given
    command_id. Returns the command result.

    """
    while 1:
        commands = ssm.list_commands(CommandId=command_id)
        if commands["Commands"][0]["Status"] in IN_PROGRESS_STATUSES:
            LOGGER.info(f"Waiting for SSM command {command_id} to complete")
            time.sleep(2)
        else:
            break

    invocation = ssm.get_command_invocation(
        CommandId=command_id,
        InstanceId=instance_id,
        PluginName="runShellScript",
    )

    if invocation["Status"] == "Success":
        return invocation["StandardOutputContent"].strip()
    else:
        raise Exception(f"SSM Command failed with Status: \"{invocation['Status']}\"")
