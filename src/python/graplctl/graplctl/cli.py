import dataclasses

from typing import Optional

import boto3
import click

from . import __version__
from . import dgraph
from . import docker_swarm

EC2 = boto3.resource("ec2")


#
# main entrypoint for grapctl
#


@dataclasses.dataclass
class GraplctlState:
    grapl_region: str
    grapl_prefix: str
    grapl_version: str


@click.group()
@click.version_option(version=__version__)
@click.option(
    "-r",
    "--grapl-region",
    type=click.Choice(docker_swarm.REGION_TO_AMI_ID.keys()),
    envvar="GRAPL_REGION",
    help="grapl region to target [$GRAPL_REGION]",
)
@click.option(
    "-p",
    "--grapl-prefix",
    type=click.STRING,
    envvar="GRAPL_CDK_DEPLOYMENT_NAME",
    help="grapl deployment name [$GRAPL_CDK_DEPLOYMENT_NAME]",
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
    ctx: click.Context, grapl_region: str, grapl_prefix: str, grapl_version: str
) -> None:
    ctx.obj = GraplctlState(grapl_region, grapl_prefix, grapl_version)


#
# swarm operational commands
#


@main.group()
def swarm():
    pass


@swarm.command(help="Spin up EC2 instances and join them together as a docker swarm cluster.")
@click.option(
    "-m", "--num-managers", type=click.INT, help="number of manager nodes to create"
)
@click.option(
    "-w", "--num-workers", type=click.INT, help="number of worker nodes to create"
)
@click.option(
    "-t", "--instance-type", type=click.STRING, help="EC2 instance type for swarm nodes"
)
@click.pass_obj
def create(
    graplctl_state: GraplctlState,
    num_managers: int,
    num_workers: int,
    instance_type: str,
) -> None:
    ami_id = docker_swarm.REGION_TO_AMI_ID[graplctl_state.grapl_region.lower()]
    prefix = graplctl_state.grapl_prefix.lower()
    version = graplctl_state.grapl_version.lower()
    manager_instances = docker_swarm.create_instances(
        ec2=EC2,
        tags={
            "grapl-deployment-name": f"{prefix}",
            "grapl-version": f"{version}",
            "grapl-swarm-role": "swarm-manager",
        },
        ami_id=ami_id,
        count=num_managers,
        instance_type=instance_type,
    )
    worker_instances = docker_swarm.create_instances(
        ec2=EC2,
        tags={
            "grapl-deployment-name": f"{prefix}",
            "grapl-version": f"{version}",
            "grapl-swarm-role": "swarm-worker",
        },
        ami_id=ami_id,
        count=num_workers,
        instance_type=instance_type,
    )


@swarm.command(help="Returns the instance ID of a docker swarm manager.")
@click.pass_obj
def manager(graplctl_state: GraplctlState):
    manager = next(
        docker_swarm.manager_instances(
            ec2=EC2,
            prefix=graplctl_state.grapl_prefix,
            version=graplctl_state.grapl_version,
        )
    )
    click.echo(manager["instance_id"])


@swarm.command(help="Tear down the docker swarm cluster and terminate all the instances")
def destroy():
    pass


@swarm.command(help="Scale up the docker swarm cluster")
def scale():
    pass


#
# dgraph operational commands
#


@main.group()
def dgraph():
    pass


@dgraph.command()
def init():
    pass


@dgraph.command()
def destroy():
    pass
