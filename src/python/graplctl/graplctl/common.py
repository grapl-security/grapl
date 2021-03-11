from __future__ import annotations

import dataclasses
import logging
import os
import sys
import time
from typing import TYPE_CHECKING, Dict, Iterator, List, Tuple

import click

if TYPE_CHECKING:
    import mypy_boto3_ec2.service_resource as ec2_resources
    from mypy_boto3_cloudwatch.client import CloudWatchClient
    from mypy_boto3_ec2 import EC2ServiceResource
    from mypy_boto3_route53 import Route53Client
    from mypy_boto3_sns import SNSClient
    from mypy_boto3_ssm.client import SSMClient

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "INFO"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

IN_PROGRESS_STATUSES = {
    "Pending",
    "InProgress",
    "Delayed",
}


@dataclasses.dataclass
class GraplctlState:
    grapl_region: str
    grapl_deployment_name: str
    grapl_version: str
    aws_profile: str
    ec2: EC2ServiceResource
    ssm: SSMClient
    cloudwatch: CloudWatchClient
    sns: SNSClient
    route53: Route53Client


# Prefer this to `pass_obj`
pass_graplctl_state = click.make_pass_decorator(GraplctlState)


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


def get_command_results(
    ssm: SSMClient, command_id: str, instance_ids: List[str]
) -> Iterator[Tuple[str, str]]:
    """Poll until the command result is available for the given
    command_id. Yields the tuple (instance_id, result) from each
    instance.

    """
    LOGGER.info(f"waiting for ssm command {command_id} to complete")
    while 1:
        commands = ssm.list_commands(CommandId=command_id)
        if (
            len(commands["Commands"]) < 1
            or commands["Commands"][0]["Status"] in IN_PROGRESS_STATUSES
        ):
            time.sleep(2)
        else:
            LOGGER.info(f"ssm command {command_id} is complete")
            break

    for instance_id in instance_ids:
        invocation = ssm.get_command_invocation(
            CommandId=command_id,
            InstanceId=instance_id,
            PluginName="runShellScript",
        )

        if invocation["Status"] == "Success":
            yield instance_id, invocation["StandardOutputContent"].strip()
        else:
            LOGGER.error(
                f"command {command_id} instance {instance_id}: {invocation['StandardErrorContent']}"
            )
            raise Exception(
                f"ssm command {command_id} failed on instance {instance_id} with Status: \"{invocation['Status']}\""
            )
