from __future__ import annotations

import dataclasses
import time
from typing import TYPE_CHECKING, Iterator, List, Tuple

import click
from grapl_common.grapl_logger import get_module_grapl_logger

if TYPE_CHECKING:
    import mypy_boto3_ec2.service_resource as ec2_resources
    from mypy_boto3_cloudwatch.client import CloudWatchClient
    from mypy_boto3_dynamodb import DynamoDBServiceResource
    from mypy_boto3_ec2 import EC2ServiceResource
    from mypy_boto3_ec2.type_defs import TagTypeDef
    from mypy_boto3_route53 import Route53Client
    from mypy_boto3_s3 import S3Client
    from mypy_boto3_sns import SNSClient
    from mypy_boto3_sqs import SQSClient
    from mypy_boto3_ssm.client import SSMClient
    from mypy_boto3_ssm.type_defs import GetCommandInvocationResultTypeDef

LOGGER = get_module_grapl_logger(log_to_stdout=True)

IN_PROGRESS_STATUSES = {
    "Pending",
    "InProgress",
    "Delayed",
}


def ticker(n: int) -> Iterator[None]:
    for _ in range(n):
        time.sleep(1)
        yield None


@dataclasses.dataclass
class State:
    grapl_region: str
    grapl_deployment_name: str
    grapl_version: str
    schema_table: str
    schema_properties_table: str
    dynamic_session_table: str

    cloudwatch: CloudWatchClient
    dynamodb: DynamoDBServiceResource
    ec2: EC2ServiceResource
    route53: Route53Client
    s3: S3Client
    sns: SNSClient
    sqs: SQSClient
    ssm: SSMClient


# Prefer this to `pass_obj`
pass_graplctl_state = click.make_pass_decorator(State)


@dataclasses.dataclass
class Tag:
    key: str
    value: str

    def into_boto_tag_specification(self) -> TagTypeDef:
        return {"Key": self.key, "Value": self.value}

    @classmethod
    def from_boto_tag_specification(cls, tag_specification: TagTypeDef) -> Tag:
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
        invocation = get_command_invocation(
            ssm=ssm,
            command_id=command_id,
            instance_id=instance_id,
        )
        if invocation["Status"] == "Success":
            yield instance_id, invocation["StandardOutputContent"].strip()
        else:
            raise SSMException(invocation)


def get_command_invocation(
    ssm: SSMClient,
    command_id: str,
    instance_id: str,
) -> GetCommandInvocationResultTypeDef:
    """
    get-command-invocation commonly throws an awfully named exception: InvalidPluginName.
    The actual meaning? This invocation hasn't become available.
    This wrapper makes it a bit more sane to use.
    """
    LOGGER.info(
        f"retrieving invocation metadata for command {command_id} on instance {instance_id}"
    )
    while True:
        try:
            result = ssm.get_command_invocation(
                CommandId=command_id,
                InstanceId=instance_id,
                PluginName="runShellScript",
            )
            LOGGER.info(
                f"retrieved invocation metadata for command {command_id} on instance {instance_id}"
            )
            return result
        except ssm.exceptions.InvalidPluginName:
            LOGGER.warn(
                f"waiting for invocation metadata to become available for command {command_id} on instance {instance_id}"
            )
            time.sleep(2)


class SSMException(Exception):
    def __init__(self, invocation: GetCommandInvocationResultTypeDef) -> None:
        msg = "\n".join(
            [
                "",
                f"STDERR: {invocation['StandardErrorContent']}",
                f"STDOUT: {invocation['StandardOutputContent']}",
                f"TO DEBUG: Try: aws ssm start-session --target {invocation['InstanceId']}",
            ]
        )
        super(SSMException, self).__init__(msg)
        self.invocation = invocation
