import dataclasses
import os
import uuid

from typing import List

from mypy_boto3_ec2 import EC2ServiceResource
from mypy_boto3_cloudwatch.client import CloudWatchClient
from mypy_boto3_sns.client import SNSClient
from mypy_boto3_ssm import SSMClient
from mypy_boto3_route53 import Route53Client

import boto3
import click

from . import __version__
from . import dgraph_ops
from . import docker_swarm_ops
from . import common

Tag = common.Tag

SESSION = boto3.Session(profile_name=os.getenv("AWS_PROFILE", "default"))

EC2: EC2ServiceResource = SESSION.resource("ec2", region_name=os.getenv("AWS_REGION"))
SSM: SSMClient = SESSION.client("ssm")
CLOUDWATCH: CloudWatchClient = SESSION.client(
    "cloudwatch", region_name=os.getenv("AWS_REGION")
)
SNS: SNSClient = SESSION.client("sns", region_name=os.getenv("AWS_REGION"))
ROUTE53: Route53Client = SESSION.client("route53", region_name=os.getenv("AWS_REGION"))

#
# main entrypoint for grapctl
#


@dataclasses.dataclass
class GraplctlState:
    grapl_region: str
    grapl_deployment_name: str
    grapl_version: str


@click.group()
@click.version_option(version=__version__)
@click.option(
    "-r",
    "--grapl-region",
    type=click.Choice(docker_swarm_ops.REGION_TO_AMI_ID.keys()),
    envvar="GRAPL_REGION",
    help="grapl region to target [$GRAPL_REGION]",
)
@click.option(
    "-n",
    "--grapl-deployment-name",
    type=click.STRING,
    envvar="GRAPL_DEPLOYMENT_NAME",
    help="grapl deployment name [$GRAPL_DEPLOYMENT_NAME]",
)
@click.option(
    "-g",
    "--grapl-version",
    type=click.STRING,
    envvar="GRAPL_VERSION",
    help="grapl version [$GRAPL_VERSION]",
)
@click.pass_context
def main(
    ctx: click.Context, grapl_region: str, grapl_deployment_name: str, grapl_version: str
) -> None:
    ctx.obj = GraplctlState(grapl_region, grapl_deployment_name, grapl_version)


#
# swarm operational commands
#


@main.group(help="commands for operating docker swarm clusters")
def swarm():
    pass


@swarm.command(
    help="start EC2 instances and join them as a docker swarm cluster",
    name="create",
)
@click.option(
    "-m",
    "--num-managers",
    type=click.IntRange(min=1, max=None),
    help="number of manager nodes to create",
    required=True,
)
@click.option(
    "-w",
    "--num-workers",
    type=click.IntRange(min=1, max=None),
    help="number of worker nodes to create",
    required=True,
)
@click.option(
    "-t",
    "--instance-type",
    type=click.STRING,
    help="EC2 instance type for swarm nodes",
    required=True,
)
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique ID for this swarm cluster (random default)",
    default=str(uuid.uuid4()),
)
@click.pass_obj
def create_swarm(
    graplctl_state: GraplctlState,
    num_managers: int,
    num_workers: int,
    instance_type: str,
    swarm_id: str,
) -> None:
    ami_id = docker_swarm_ops.REGION_TO_AMI_ID[graplctl_state.grapl_region.lower()]
    security_group_id = docker_swarm_ops.swarm_security_group_id(
        ec2=EC2,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    vpc_id = docker_swarm_ops.swarm_vpc_id(
        ec2=EC2, swarm_security_group_id=security_group_id
    )

    click.echo(f"retrieving subnet IDs in vpc {vpc_id}")
    subnet_ids = set(
        docker_swarm_ops.subnet_ids(
            ec2=EC2, swarm_vpc_id=vpc_id, deployment_name=graplctl_state.grapl_deployment_name
        )
    )
    click.echo(f"retrieved subnet IDs in vpc {vpc_id}")

    click.echo(f"creating manager instances in vpc {vpc_id}")
    manager_instances = docker_swarm_ops.create_instances(
        ec2=EC2,
        ssm=SSM,
        deployment_name=graplctl_state.grapl_deployment_name,
        region=graplctl_state.grapl_region,
        version=graplctl_state.grapl_version,
        swarm_manager=True,
        swarm_id=swarm_id,
        ami_id=ami_id,
        count=num_managers,
        instance_type=instance_type,
        security_group_id=security_group_id,
        subnet_ids=subnet_ids,
    )
    manager_instance_ids_str = ",".join(w.instance_id for w in manager_instances)
    click.echo(f"created manager instances {manager_instance_ids_str} in vpc {vpc_id}")

    click.echo(f"initializing manager instances {manager_instance_ids_str}")
    docker_swarm_ops.init_instances(
        ssm=SSM,
        deployment_name=graplctl_state.grapl_deployment_name,
        instances=manager_instances,
    )
    click.echo(f"initialized manager instances {manager_instance_ids_str}")

    manager_instance = manager_instances[0]
    click.echo(
        f"configuring docker swarm cluster manager {manager_instance.instance_id}"
    )
    docker_swarm_ops.init_docker_swarm(
        ec2=EC2,
        ssm=SSM,
        deployment_name=graplctl_state.grapl_deployment_name,
        manager_instance=manager_instance,
    )
    click.echo(
        f"configured docker swarm cluster manager {manager_instance.instance_id}"
    )

    if len(manager_instances) > 1:
        click.echo(
            f"extracting docker swarm manager join token from manager {manager_instance.instance_id}"
        )
        manager_join_token = docker_swarm_ops.extract_join_token(
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            manager_instance=manager_instance,
            manager=True,
        )
        click.echo(
            f"extracted docker swarm manager join token from manager {manager_instance.instance_id}"
        )

        remaining_manager_instance_ids_str = ",".join(
            w.instance_id for w in manager_instances[1:]
        )
        click.echo(
            f"joining docker swarm manager instances {remaining_manager_instance_ids_str}"
        )
        docker_swarm_ops.join_swarm_nodes(
            ec2=EC2,
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            instances=manager_instances[1:],
            join_token=manager_join_token,
            manager=True,
            manager_ip=manager_instance.private_ip_address,
        )
        click.echo(
            f"joined docker swarm manager instances {remaining_manager_instance_ids_str}"
        )

    click.echo(f"creating worker instances in vpc {vpc_id}")
    worker_instances = docker_swarm_ops.create_instances(
        ec2=EC2,
        ssm=SSM,
        deployment_name=graplctl_state.grapl_deployment_name,
        region=graplctl_state.grapl_region,
        version=graplctl_state.grapl_version,
        swarm_manager=False,
        swarm_id=swarm_id,
        ami_id=ami_id,
        count=num_workers,
        instance_type=instance_type,
        security_group_id=security_group_id,
        subnet_ids=subnet_ids,
    )
    worker_instance_ids_str = ",".join(w.instance_id for w in worker_instances)
    click.echo(f"created worker instances {worker_instance_ids_str} in vpc {vpc_id}")

    click.echo(f"initializing worker instances {worker_instance_ids_str}")
    docker_swarm_ops.init_instances(
        ssm=SSM,
        deployment_name=graplctl_state.grapl_deployment_name,
        instances=worker_instances,
    )
    click.echo(f"initialized worker instances {worker_instance_ids_str}")

    click.echo(
        "extracting docker swarm worker join token from manager {manager_instance.instance_id}"
    )
    worker_join_token = docker_swarm_ops.extract_join_token(
        ssm=SSM,
        deployment_name=graplctl_state.grapl_deployment_name,
        manager_instance=manager_instance,
        manager=False,
    )
    click.echo(
        "extracted docker swarm worker join token from manager {manager_instance.instance_id}"
    )

    click.echo(f"joining docker swarm worker instances {worker_instance_ids_str}")
    docker_swarm_ops.join_swarm_nodes(
        ec2=EC2,
        ssm=SSM,
        deployment_name=graplctl_state.grapl_deployment_name,
        instances=worker_instances,
        join_token=worker_join_token,
        manager=False,
        manager_ip=manager_instance.private_ip_address,
    )
    click.echo(f"joined docker swarm worker instances {worker_instance_ids_str}")


@swarm.command(help="list swarm IDs for each of the swarm clusters")
@click.pass_obj
def ls(graplctl_state: GraplctlState):
    for swarm_id in docker_swarm_ops.swarm_ids(
        ec2=EC2,
        deployment_name=graplctl_state.grapl_deployment_name,
        region=graplctl_state.grapl_region,
        version=graplctl_state.grapl_version,
    ):
        click.echo(swarm_id)


@swarm.command(help="get instance IDs for a docker swarm's managers")
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique ID of the swarm cluster",
    required=True,
)
@click.pass_obj
def managers(graplctl_state: GraplctlState, swarm_id: str):
    for manager_instance in docker_swarm_ops.swarm_instances(
        ec2=EC2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
        swarm_id=swarm_id,
        swarm_manager=True,
    ):
        click.echo(manager_instance.instance_id)


@swarm.command(help="terminate a docker swarm cluster's instances")
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique ID of the swarm cluster",
    required=True,
)
@click.confirmation_option(prompt="are you sure you want to destroy the swarm cluster?")
@click.pass_obj
def destroy(graplctl_state: GraplctlState, swarm_id: str):
    for instance in docker_swarm_ops.swarm_instances(
        ec2=EC2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
        swarm_id=swarm_id,
    ):
        EC2.Instance(instance.instance_id).terminate(InstanceIds=[instance.instance_id])
        click.echo(f"terminated instance {instance.instance_id}")


@swarm.command(name="exec", help="execute a command on a swarm manager")
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique ID of the swarm cluster",
    required=True,
)
@click.argument("command", nargs=-1, type=click.STRING)
@click.pass_obj
def exec_(graplctl_state: GraplctlState, swarm_id: str, command: List[str]):
    click.echo(
        docker_swarm_ops.exec_(
            ec2=EC2,
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            region=graplctl_state.grapl_region,
            version=graplctl_state.grapl_version,
            swarm_id=swarm_id,
            command=command,
        )
    )


@swarm.command(help="scale up a docker swarm cluster")
@click.option(
    "-m",
    "--num-managers",
    type=click.IntRange(min=0, max=None),
    help="number of additional manager nodes to create",
    default=0,
)
@click.option(
    "-w",
    "--num-workers",
    type=click.IntRange(min=0, max=None),
    help="number of additional worker nodes to create",
    default=0,
)
@click.option(
    "-t",
    "--instance-type",
    type=click.STRING,
    help="EC2 instance type for swarm nodes",
    required=True,
)
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique ID of the swarm cluster",
    required=True,
)
@click.pass_obj
def scale(
    graplctl_state: GraplctlState,
    num_managers: int,
    num_workers: int,
    instance_type: str,
    swarm_id: str,
):
    security_group_id = docker_swarm_ops.swarm_security_group_id(
        ec2=EC2,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    vpc_id = docker_swarm_ops.swarm_vpc_id(
        ec2=EC2, swarm_security_group_id=security_group_id
    )
    subnet_ids = set(
        docker_swarm_ops.subnet_ids(
            ec2=EC2,
            swarm_vpc_id=vpc_id,
            deployment_name=graplctl_state.grapl_deployment_name,
        )
    )
    manager_instance = next(
        docker_swarm_ops.swarm_instances(
            ec2=EC2,
            deployment_name=graplctl_state.grapl_deployment_name,
            version=graplctl_state.grapl_version,
            region=graplctl_state.grapl_region,
            swarm_id=swarm_id,
            swarm_manager=True,
        )
    )

    if num_managers > 0:
        click.echo(f"creating manager instances in vpc {vpc_id}")
        manager_instances = docker_swarm_ops.create_instances(
            ec2=EC2,
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            region=graplctl_state.grapl_region,
            version=graplctl_state.grapl_version,
            swarm_manager=True,
            swarm_id=swarm_id,
            ami_id=docker_swarm_ops.REGION_TO_AMI_ID[graplctl_state.grapl_region],
            count=num_managers,
            instance_type=instance_type,
            security_group_id=security_group_id,
            subnet_ids=subnet_ids,
        )
        manager_instance_ids_str = ",".join(i.instance_id for i in manager_instances)
        click.echo(
            f"created manager instances {manager_instance_ids_str} in vpc {vpc_id}"
        )

        click.echo(f"initializing manager instances {manager_instance_ids_str}")
        docker_swarm_ops.init_instances(
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            instances=manager_instances,
        )
        click.echo(f"initialized manager instances {manager_instance_ids_str}")

        click.echo(
            f"extracting docker swarm manager join token from manager {manager_instance.instance_id}"
        )
        manager_join_token = docker_swarm_ops.extract_join_token(
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            manager_instance=manager_instance,
            manager=True,
        )
        click.echo(
            f"extracted docker swarm manager join token from manager {manager_instance.instance_id}"
        )

        click.echo(f"joining docker swarm manager instances {manager_instance_ids_str}")
        docker_swarm_ops.join_swarm_nodes(
            ec2=EC2,
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            instances=manager_instances,
            join_token=manager_join_token,
            manager=True,
            manager_ip=manager_instance.private_ip_address,
        )
        click.echo(f"joined docker swarm manager instances {manager_instance_ids_str}")

    if num_workers > 0:
        click.echo(f"creating worker instances in vpc {vpc_id}")
        worker_instances = docker_swarm_ops.create_instances(
            ec2=EC2,
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            region=graplctl_state.grapl_region,
            version=graplctl_state.grapl_version,
            swarm_manager=False,
            swarm_id=swarm_id,
            ami_id=docker_swarm_ops.REGION_TO_AMI_ID[graplctl_state.grapl_region],
            count=num_managers,
            instance_type=instance_type,
            security_group_id=security_group_id,
            subnet_ids=subnet_ids,
        )
        worker_instance_ids_str = ",".join(i.instance_id for i in worker_instances)
        click.echo(
            f"created worker instances {worker_instance_ids_str} in vpc {vpc_id}"
        )

        click.echo(f"initializing worker instances {worker_instance_ids_str}")
        docker_swarm_ops.init_instances(
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            instances=worker_instances,
        )
        click.echo(f"initialized worker instances {worker_instance_ids_str}")

        click.echo(
            "extracting docker swarm worker join token from manager {manager_instance.instance_id}"
        )
        worker_join_token = docker_swarm_ops.extract_join_token(
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            manager_instance=manager_instance,
            manager=False,
        )
        click.echo(
            "extracted docker swarm worker join token from manager {manager_instance.instance_id}"
        )

        click.echo(f"joining docker swarm worker instances {worker_instance_ids_str}")
        docker_swarm_ops.join_swarm_nodes(
            ec2=EC2,
            ssm=SSM,
            deployment_name=graplctl_state.grapl_deployment_name,
            instances=worker_instances,
            join_token=worker_join_token,
            manager=False,
            manager_ip=manager_instance.private_ip_address,
        )
        click.echo(f"joined docker swarm worker instances {worker_instance_ids_str}")


#
# dgraph operational commands
#


@main.group(help="commands for operating dgraph")
def dgraph():
    pass


@dgraph.command(
    help="spin up a swarm cluster and deploy dgraph on it",
    name="create",
)
@click.option(
    "-t",
    "--instance-type",
    type=click.Choice(choices=("i3.large", "i3.xlarge", "i3.2xlarge")),
    help="EC2 instance type for swarm nodes",
    required=True,
)
@click.pass_obj
def create_dgraph(graplctl_state: GraplctlState, instance_type: str):
    swarm_id = f"{graplctl_state.grapl_deployment_name.lower()}-dgraph-swarm"
    click.echo("creating swarm {swarm_id}")
    create_swarm(
        graplctl_state=graplctl_state,
        num_managers=1,
        num_workers=2,
        instance_type=instance_type,
        swarm_id=swarm_id,
    )
    click.echo("created swarm {swarm_id}")

    manager_instance = next(
        docker_swarm_ops.swarm_instances(
            ec2=EC2,
            deployment_name=graplctl_state.grapl_deployment_name,
            version=graplctl_state.grapl_version,
            region=graplctl_state.grapl_region,
            swarm_id=swarm_id,
            swarm_manager=True,
        )
    )

    swarm_instances = list(
        docker_swarm_ops.swarm_instances(
            ec2=EC2,
            deployment_name=graplctl_state.grapl_deployment_name,
            version=graplctl_state.grapl_version,
            region=graplctl_state.grapl_region,
            swarm_id=swarm_id,
        )
    )

    click.echo(f"configuring instances in swarm {swarm_id} for dgraph")
    dgraph_ops.init_dgraph(
        ssm=SSM,
        deployment_name=graplctl_state.grapl_deployment_name,
        instances=swarm_instances,
    )
    click.echo(f"configured instances in swarm {swarm_id} for dgraph")

    click.echo(f"creating disk usage alarms for dgraph in swarm {swarm_id}")
    for instance in swarm_instances:
        dgraph_ops.create_disk_usage_alarms(
            cloudwatch=CLOUDWATCH,
            sns=SNS,
            instance_id=instance.instance_id,
            deployment_name=graplctl_state.grapl_deployment_name,
        )
    click.echo(f"created disk usage alarms for dgraph in swarm {swarm_id}")

    click.echo(f"deploying dgraph in swarm {swarm_id}")
    dgraph_ops.deploy_dgraph(
        ssm=SSM,
        deployment_name=graplctl_state.grapl_deployment_name,
        manager_instance=manager_instance,
        worker_instances=tuple(
            instance
            for instance in swarm_instances
            if Tag(key="grapl-swarm-role", value="swarm-worker") in instance.tags
        ),
    )
    click.echo(f"deployed dgraph in swarm {swarm_id}")

    click.echo(f"updating DNS A records for dgraph in swarm {swarm_id}")
    hosted_zone_id = ROUTE53.list_hosted_zones_by_name(
        DNSName=f"{graplctl_state.grapl_deployment_name.lower()}.dgraph.grapl"
    )["HostedZones"][0]["Id"]
    for instance in docker_swarm_ops.swarm_instances(
        ec2=EC2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
        swarm_id=swarm_id,
    ):
        dgraph_ops.insert_dns_ip(
            route53=ROUTE53,
            dns_name=f"{graplctl_state.grapl_deployment_name.lower()}.dgraph.grapl",
            ip_address=instance.private_ip_address,
            hosted_zone_id=hosted_zone_id,
        )
    click.echo(f"updated DNS A records for dgraph in swarm {swarm_id}")


@dgraph.command(help="remove DGraph DNS records")
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique ID of the swarm cluster",
    required=True,
)
@click.confirmation_option(
    prompt="are you sure you want to remove the DGraph DNS records?"
)
@click.pass_obj
def remove_dns(graplctl_state: GraplctlState, swarm_id: str):
    hosted_zone_id = ROUTE53.list_hosted_zones_by_name(
        DNSName=f"{graplctl_state.grapl_deployment_name.lower()}.dgraph.grapl"
    )["HostedZones"][0]["Id"]
    for instance in docker_swarm_ops.swarm_instances(
        ec2=EC2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
        swarm_id=swarm_id,
    ):
        click.echo(f"removing DNS records for swarm {swarm_id}")
        dgraph_ops.remove_dns_ip(
            route53=ROUTE53,
            dns_name=f"{graplctl_state.grapl_deployment_name.lower()}.dgraph.grapl",
            ip_address=instance.private_ip_address,
            hosted_zone_id=hosted_zone_id,
        )
        click.echo(f"removed DNS records for swarm {swarm_id}")
