from __future__ import annotations

import base64
import itertools
import json
import logging
import os
import shlex
import sys
import time
from typing import Dict, Iterator, List, Optional, Set

from mypy_boto3_ec2 import EC2ServiceResource
from mypy_boto3_ssm import SSMClient

from . import common

get_command_results = common.get_command_results
Tag = common.Tag
Ec2Instance = common.Ec2Instance

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "INFO"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

# This mapping was compiled on 2020-10-14 by running the
# following query for each region:
#
# aws ec2 describe-images \
#   --owners amazon \
#   --filters 'Name=name,Values=amzn2-ami-hvm-2.0.????????.?-x86_64-gp2' 'Name=state,Values=available' \
#   --query 'reverse(sort_by(Images, &CreationDate))[:1]' \
#   --region us-east-1
#
# It should probably be updated periodically.
REGION_TO_AMI_ID = {
    "us-east-1": "ami-0947d2ba12ee1ff75",
    "us-east-2": "ami-03657b56516ab7912",
    "us-west-1": "ami-0e4035ae3f70c400f",
    "us-west-2": "ami-0528a5175983e7f28",
}


def swarm_security_group_id(ec2: EC2ServiceResource, deployment_name: str) -> str:
    """Return the security group ID for the swarm security group"""
    result = ec2.security_groups.filter(
        Filters=[
            {"Name": "group-name", "Values": [f"{deployment_name.lower()}-grapl-swarm"]}
        ]
    )
    return list(result)[0].group_id


def swarm_vpc_id(ec2: EC2ServiceResource, swarm_security_group_id: str) -> str:
    """Return the VPC ID for the swarm cluster"""
    return ec2.SecurityGroup(swarm_security_group_id).vpc_id


def subnet_ids(
    ec2: EC2ServiceResource, swarm_vpc_id: str, deployment_name: str
) -> Iterator[str]:
    """Yields the subnet IDs for the grapl deployment"""
    for subnet in ec2.Vpc(swarm_vpc_id).subnets.filter(
        Filters=[
            {"Name": "tag:aws-cdk:subnet-type", "Values": ["Private"]},
            {"Name": "tag:name", "Values": [f"{deployment_name.lower()}-grapl-vpc"]},
        ]
    ):
        yield subnet.subnet_id


def create_instances(
    ec2: EC2ServiceResource,
    ssm: SSMClient,
    deployment_name: str,
    region: str,
    version: str,
    swarm_manager: bool,
    swarm_id: str,
    ami_id: str,
    count: int,
    instance_type: str,
    security_group_id: str,
    subnet_ids: Set[str],
) -> List[Ec2Instance]:
    """Spin up EC2 instances. Returns a list of the instances."""
    counts = {subnet_id: 0 for subnet_id in subnet_ids}
    ids_cycle = itertools.cycle(subnet_ids)
    for _ in range(count):
        subnet_id = next(ids_cycle)
        counts[subnet_id] += 1  # distribute instances across subnets

    instances = []
    for subnet_id in subnet_ids:
        if counts[subnet_id] > 0:
            instances.extend(
                ec2.create_instances(
                    ImageId=ami_id,
                    InstanceType=instance_type,
                    MaxCount=counts[subnet_id],
                    MinCount=counts[subnet_id],
                    TagSpecifications=[
                        {
                            "ResourceType": "instance",
                            "Tags": [
                                t.into_boto_tag_specification()
                                for t in [
                                    Tag(
                                        key="grapl-deployment-name",
                                        value=f"{deployment_name.lower()}",
                                    ),
                                    Tag(
                                        key="grapl-version", value=f"{version.lower()}"
                                    ),
                                    Tag(key="grapl-region", value=f"{region.lower()}"),
                                    Tag(
                                        key="grapl-swarm-role",
                                        value="swarm-manager"
                                        if swarm_manager
                                        else "swarm-worker",
                                    ),
                                    Tag(key="grapl-swarm-id", value=swarm_id),
                                ]
                            ],
                        }
                    ],
                    SecurityGroupIds=[security_group_id],
                    SubnetId=subnet_id,
                    IamInstanceProfile={
                        "Name": f"{deployment_name.lower()}-swarm-instance-profile"
                    },
                    UserData=base64.b64encode(
                        b"#!/bin/bash\nsleep 30\nyum install -y python3\n"
                    ).decode("utf-8"),
                )
            )

    for instance in instances:
        LOGGER.info(f'waiting for instance {instance.instance_id} to report "running"')
        while instance.state["Name"].lower() != "running":
            time.sleep(2)
            instance.load()
        LOGGER.info(f'instance {instance.instance_id} is "running"')

    for instance in instances:
        LOGGER.info(
            f'waiting for instance {instance.instance_id} to report SSM PingStatus "Online"'
        )
        while 1:
            instance_information = ssm.describe_instance_information(
                Filters=[{"Key": "InstanceIds", "Values": [instance.instance_id]}]
            )["InstanceInformationList"]
            if (
                len(instance_information) < 1
                or instance_information[0]["PingStatus"] != "Online"
            ):
                time.sleep(2)
            elif instance_information[0]["PingStatus"] == "Online":
                break
        LOGGER.info(f'instance {instance.instance_id} is "Online"')

    return [Ec2Instance.from_boto_instance(instance) for instance in instances]


def swarm_instances(
    ec2: EC2ServiceResource,
    deployment_name: Optional[str] = None,
    version: Optional[str] = None,
    region: Optional[str] = None,
    swarm_id: Optional[str] = None,
    swarm_manager: Optional[bool] = None,
) -> Iterator[Ec2Instance]:
    """Yields all the swarm instances in this grapl deployment"""
    tags = []
    if deployment_name is not None:
        tags.append(Tag(key="grapl-deployment-name", value=deployment_name))
    if version is not None:
        tags.append(Tag(key="grapl-version", value=version))
    if region is not None:
        tags.append(Tag(key="grapl-region", value=region))
    if swarm_id is not None:
        tags.append(Tag(key="grapl-swarm-id", value=swarm_id))
    if swarm_manager is not None:
        tags.append(
            Tag(
                key="grapl-swarm-role",
                value="swarm-manager" if swarm_manager else "swarm-worker",
            )
        )

    filters = [{"Name": f"tag:{t.key}", "Values": [t.value]} for t in tags]
    filters.append({"Name": "instance-state-name", "Values": ["running"]})

    for instance in ec2.instances.filter(Filters=filters):
        yield Ec2Instance.from_boto_instance(instance)


def swarm_ids(
    ec2: EC2ServiceResource, deployment_name: str, version: str, region: str
) -> Set[str]:
    """Returns the unique swarm IDs in this grapl deployment."""
    ids = set()
    for instance in swarm_instances(
        ec2=ec2,
        deployment_name=deployment_name,
        version=version,
        region=region,
        swarm_manager=True,
    ):
        for tag in instance.tags:
            if tag.key == "grapl-swarm-id":
                ids.add(tag.value)
    return ids


def init_instances(
    ssm: SSMClient, deployment_name: str, instances: List[Ec2Instance]
) -> None:
    """Initialize the EC2 instances"""
    instance_ids = [instance.instance_id for instance in instances]
    command = ssm.send_command(
        InstanceIds=instance_ids,
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{deployment_name.lower()}-swarm-config-bucket/instance_init.py"
                    }
                )
            ],
            "commandLine": ["sleep 60 && /usr/bin/python3 instance_init.py"],
        },
    )
    command_id = command["Command"]["CommandId"]
    for instance_id, result in get_command_results(ssm, command_id, instance_ids):
        LOGGER.info(f"command {command_id} instance {instance_id}: {result}")


def init_docker_swarm(
    ec2: EC2ServiceResource,
    ssm: SSMClient,
    deployment_name: str,
    manager_instance: Ec2Instance,
) -> None:
    """Initialize the docker swarm cluster"""
    command = ssm.send_command(
        InstanceIds=[manager_instance.instance_id],
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{deployment_name.lower()}-swarm-config-bucket/swarm_init.py"
                    }
                )
            ],
            "commandLine": ["/usr/bin/python3 swarm_init.py"],
        },
    )
    command_id = command["Command"]["CommandId"]
    instance_id, result = next(
        get_command_results(ssm, command_id, [manager_instance.instance_id])
    )
    LOGGER.info(f"command {command_id} instance {instance_id}: {result}")
    LOGGER.info(f"configured instance {manager_instance.instance_id} as swarm manager")

    ec2.create_tags(
        Resources=[manager_instance.instance_id],
        Tags=[{"Key": "grapl-swarm-role", "Value": "swarm-manager"}],
    )


def extract_join_token(
    ssm: SSMClient,
    deployment_name: str,
    manager_instance: Ec2Instance,
    manager=False,
) -> str:
    """Returns the join token for the swarm cluster"""
    command = ssm.send_command(
        InstanceIds=[manager_instance.instance_id],
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{deployment_name.lower()}-swarm-config-bucket/swarm_token.py"
                    }
                )
            ],
            "commandLine": [f"/usr/bin/python3 swarm_token.py {str(manager).lower()}"],
        },
    )
    command_id = command["Command"]["CommandId"]
    LOGGER.info(f"extracted join token from instance {manager_instance.instance_id}")
    return next(get_command_results(ssm, command_id, [manager_instance.instance_id]))[1]


def join_swarm_nodes(
    ec2: EC2ServiceResource,
    ssm: SSMClient,
    deployment_name: str,
    instances: List[Ec2Instance],
    join_token: str,
    manager: bool,
    manager_ip: str,
) -> None:
    """Join nodes to the swarm cluster"""
    instance_ids = [instance.instance_id for instance in instances]
    command = ssm.send_command(
        InstanceIds=instance_ids,
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{deployment_name.lower()}-swarm-config-bucket/swarm_join.py"
                    }
                )
            ],
            "commandLine": [
                f"/usr/bin/python3 swarm_join.py {join_token} {manager_ip}"
            ],
        },
    )
    command_id = command["Command"]["CommandId"]
    for instance_id, result in get_command_results(ssm, command_id, instance_ids):
        LOGGER.info(f"command {command_id} instance {instance_id}: {result}")
    LOGGER.info(f"joined instances {','.join(instance_ids)} to docker swarm cluster")

    ec2.create_tags(
        Resources=instance_ids,
        Tags=[
            {
                "Key": "grapl-swarm-role",
                "Value": "swarm-manager" if manager else "swarm-worker",
            }
        ],
    )


def restart_daemons(
    ssm: SSMClient, deployment_name: str, instances: List[Ec2Instance]
) -> None:
    """Restart the cloudwatch agent and docker daemon"""
    instance_ids = [instance.instance_id for instance in instances]
    command = ssm.send_command(
        InstanceIds=instance_ids,
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{deployment_name.lower()}-swarm-config-bucket/restart_daemons.py"
                    }
                )
            ],
            "commandLine": ["/usr/bin/python3 restart_daemons.py"],
        },
    )
    command_id = command["Command"]["CommandId"]
    for instance_id, result in get_command_results(ssm, command_id, instance_ids):
        LOGGER.info(f"command {command_id} instance {instance_id}: {result}")
    LOGGER.info(
        f"restarted cloudwatch agent and docker daemon on instances {','.join(instance_ids)}"
    )


def configure_docker_daemon(
    ssm: SSMClient,
    deployment_name: str,
    instances: List[Ec2Instance],
    config: Dict,
) -> None:
    """Write the given daemon configuration to /etc/docker/daemon.json"""
    instance_ids = [instance.instance_id for instance in instances]
    command = ssm.send_command(
        InstanceIds=instance_ids,
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{deployment_name.lower()}-swarm-config-bucket/configure_docker_daemon.py"
                    }
                )
            ],
            "commandLine": [
                f"/usr/bin/python3 configure_docker_daemon.py {shlex.quote(json.dumps(config, separators=(',', ':')))}"
            ],
        },
    )
    command_id = command["Command"]["CommandId"]
    for instance_id, result in get_command_results(ssm, command_id, instance_ids):
        LOGGER.info(f"command {command_id} instance {instance_id}: {result}")
    LOGGER.info(f"configured docker daemon on instances {','.join(instance_ids)}")


def exec_(
    ec2: EC2ServiceResource,
    ssm: SSMClient,
    deployment_name: str,
    version: str,
    region: str,
    swarm_id: str,
    command: List[str],
) -> str:
    """Execute the given command on the swarm manager. Returns the result."""
    manager_instance = next(
        swarm_instances(
            ec2=ec2,
            deployment_name=deployment_name,
            version=version,
            region=region,
            swarm_id=swarm_id,
            swarm_manager=True,
        )
    )
    encoded_command = base64.b64encode(
        bytes(",".join(c for c in command), "utf-8")
    ).decode("utf-8")
    ssm_command = ssm.send_command(
        InstanceIds=[manager_instance.instance_id],
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{deployment_name.lower()}-swarm-config-bucket/swarm_exec.py"
                    }
                )
            ],
            "commandLine": [f"/usr/bin/python3 swarm_exec.py {encoded_command}"],
        },
    )
    ssm_command_id = ssm_command["Command"]["CommandId"]
    return next(
        get_command_results(ssm, ssm_command_id, [manager_instance.instance_id])
    )[1]
