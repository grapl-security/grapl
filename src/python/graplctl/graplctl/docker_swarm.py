from __future__ import annotations

import json
import logging
import os
import sys

from typing import TYPE_CHECKING, Any, Dict, Iterator, List, NamedTuple, Optional

import boto3
from botocore.exceptions import ClientError

if TYPE_CHECKING:
    from mypy_boto3_ec2 import EC2ServiceResource
    from mypy_boto3_cloudwatch.client import CloudWatchClient
    from mypy_boto3_cloudwatch.type_defs import MetricTypeDef
    from mypy_boto3_sns.client import SNSClient


LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "INFO"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


REGION_TO_AMI_ID = {
    "us-east-1": "ami-0947d2ba12ee1ff75",
    "us-east-2": "ami-03657b56516ab7912",
    "us-west-1": "ami-0e4035ae3f70c400f",
    "us-west-2": "ami-0528a5175983e7f28",
}

InstanceTuple = NamedTuple(
    "InstanceTuple",
    (
        ("instance_id", str),
        ("private_ip_address", str),
        ("private_dns_name", str),
    ),
)


def create_instances(
    ec2: EC2ServiceResource,
    tags: Dict[str, str],
    ami_id: str,
    count: int,
    instance_type: str,
) -> List[InstanceTuple]:
    ec2.create_instances(
        ImageId=ami_id,
        MaxCount=count,
        MinCount=count,
        TagSpecifications=[
            {
                "ResourceType": "instance",
                "Tags": [{"Key": k, "Value": v} for k, v in tags.items()],
            }
        ],
    )


def manager_instances(ec2: Any, prefix: str, version: str) -> Iterator[InstanceTuple]:
    pass


def _swarm_instances(ec2: Any) -> Iterator[InstanceTuple]:
    """Yields tuples of (instance_id, private_ip, hostname) for all the
    instances in the SwarmASG.

    """
    result = ec2.describe_instances(
        Filters=[{"Name": "tag:Name", "Values": ["grapl-swarm"]}]
    )
    for reservation in result["Reservations"]:
        for instance in reservation["Instances"]:
            if instance["State"]["Name"] != "terminated":
                yield InstanceTuple(
                    instance_id=instance["InstanceId"],
                    private_ip_address=instance["PrivateIpAddress"],
                    private_dns_name=instance["PrivateDnsName"],
                )


def _instance_ip_address(instance_id: str) -> str:
    ec2 = boto3.resource("ec2")
    instance = ec2.Instance(instance_id)
    return instance.private_ip_address


def _dns_ip_addresses(
    route53: Any, dns_name: str, ip_address: Optional[str], hosted_zone_id: str
) -> Iterator[str]:
    for rrset in route53.list_resource_record_sets(
        HostedZoneId=hosted_zone_id,
        StartRecordName=dns_name,
    )["ResourceRecordSets"]:
        if rrset["Type"] == "A":
            for rrecord in rrset["ResourceRecords"]:
                yield rrecord["Value"]
    if ip_address is not None:
        yield ip_address


def _remove_dns_ip(dns_name: str, ip_address: str, hosted_zone_id: str) -> None:
    route53 = boto3.client("route53")
    ip_addresses = [
        ip
        for ip in _dns_ip_addresses(route53, dns_name, None, hosted_zone_id)
        if ip != ip_address
    ]

    change = {
        "Action": "DELETE",  # delete the A record if this is the last address
        "ResourceRecordSet": {
            "Name": dns_name,
            "Type": "A",
            "TTL": 300,
            "ResourceRecords": [{"Value": ip_address}],
        },
    }
    if len(ip_addresses) > 0:
        change["Action"] = "UPSERT"
        change["ResourceRecordSet"]["ResourceRecords"] = [
            {"Value": ip} for ip in ip_addresses
        ]

    try:
        comment = f"Removed {ip_address} from {dns_name} DNS A Record"
        route53.change_resource_record_sets(
            HostedZoneId=hosted_zone_id,
            ChangeBatch={
                "Changes": [change],
                "Comment": comment,
            },
        )
        LOGGER.info(comment)
    except ClientError as e:
        if e.response["Error"]["Code"] == "InvalidChangeBatch":
            LOGGER.warn(f"DNS record does not exist for {ip_address}")
        else:
            raise e


def _insert_dns_ip(dns_name: str, ip_address: str, hosted_zone_id: str) -> None:
    route53 = boto3.client("route53")
    comment = f"Inserted {ip_address} into {dns_name} DNS A Record"
    route53.change_resource_record_sets(
        HostedZoneId=hosted_zone_id,
        ChangeBatch={
            "Changes": [
                {
                    "Action": "UPSERT",
                    "ResourceRecordSet": {
                        "Name": dns_name,
                        "Type": "A",
                        "TTL": 300,
                        "ResourceRecords": [
                            {"Value": ip}
                            for ip in _dns_ip_addresses(
                                route53, dns_name, ip_address, hosted_zone_id
                            )
                        ],
                    },
                },
            ],
            "Comment": comment,
        },
    )
    LOGGER.info(comment)


def _init_docker_swarm(
    ec2: Any,
    ssm: Any,
    cloudwatch: CloudWatchClient,
    prefix: str,
    manager_id: str,
    manager_ip: str,
    manager_hostname: str,
) -> str:
    """Initialize the docker swarm manager. Returns the join token
    necessary to attach workers to the swarm.

    """
    _create_disk_usage_alarms(cloudwatch, manager_id, prefix)
    command = ssm.send_command(
        # Targets=[{"Key": "tag:Name", "Values": ["Grapl/swarm/SwarmCluster/SwarmASG"]}],
        InstanceIds=[manager_id],
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{prefix.lower()}-swarm-config-bucket/swarm_init.py"
                    }
                )
            ],
            "commandLine": ["python3 swarm_init.py"],
        },
    )
    command_id = command["Command"]["CommandId"]
    result = _get_command_result(ssm, command_id, manager_id)
    ec2.create_tags(
        Resources=[manager_id],
        Tags=[{"Key": "grapl-swarm-role", "Value": "swarm-manager"}],
    )
    LOGGER.info(
        f"Instance {manager_id} with IP {manager_ip} and hostname {manager_hostname} is the docker swarm cluster manager"
    )
    return result


def _join_worker_nodes(
    ec2: Any,
    ssm: Any,
    cloudwatch: CloudWatchClient,
    prefix: str,
    instances: Iterator[InstanceTuple],
    join_token: str,
    manager_ip: str,
) -> List[str]:
    """Join worker nodes to the swarm cluster. Returns hostnames of the
    worker nodes."""
    hostnames = []
    for instance_id, _, hostname in instances:
        _create_disk_usage_alarms(cloudwatch, instance_id, prefix)
        command = ssm.send_command(
            # Targets=[{"Key": "tag:Name", "Values": ["Grapl/swarm/SwarmCluster/SwarmASG"]}],
            InstanceIds=[instance_id],
            DocumentName="AWS-RunRemoteScript",
            Parameters={
                "sourceType": ["S3"],
                "sourceInfo": [
                    json.dumps(
                        {
                            "path": f"https://s3.amazonaws.com/{prefix.lower()}-swarm-config-bucket/swarm_join.py"
                        }
                    )
                ],
                "commandLine": [f"python3 swarm_join.py {join_token} {manager_ip}"],
            },
        )
        command_id = command["Command"]["CommandId"]
        _get_command_result(ssm, command_id, instance_id)
        ec2.create_tags(
            Resources=[instance_id],
            Tags=[{"Key": "grapl-swarm-role", "Value": "swarm-worker"}],
        )
        LOGGER.info(
            f"Joined worker instance {instance_id} with hostname {hostname} to the docker swarm cluster"
        )
        hostnames.append(hostname)
    return hostnames
