from __future__ import annotations

import pathlib
from os import PathLike
from typing import TYPE_CHECKING

import click
import graplctl.aws.lib as aws_cdk_ops
import graplctl.dgraph.lib as dgraph_ops
import graplctl.swarm.lib as docker_swarm_ops
from grapl_common.utils.find_grapl_root import find_grapl_root
from graplctl.common import State, pass_graplctl_state

if TYPE_CHECKING:
    from mypy_boto3_ec2.literals import InstanceTypeType


#
# aws deployment & provisioning commands
#


@click.group()
@click.pass_context
@pass_graplctl_state
def aws(
    graplctl_state: State,
    ctx: click.Context,
) -> None:
    """commands for managing grapl aws resources"""
    pass


@aws.command()
@click.option(
    "-a", "--all", is_flag=True, required=True, help="all services and resources"
)
@click.option(
    "-t",
    "--dgraph-instance-type",
    type=click.Choice(choices=("i3.large", "i3.xlarge", "i3.2xlarge")),
    help="ec2 instance type for dgraph swarm nodes",
    required=True,
)
@click.argument(
    "grapl_root",
    type=click.Path(exists=True, file_okay=False, resolve_path=True),
    required=True,
    default=find_grapl_root(),
)
@click.confirmation_option(prompt=f"this will incur aws charges, ok?")
@pass_graplctl_state
def deploy(
    graplctl_state: State,
    all: bool,
    dgraph_instance_type: InstanceTypeType,
    grapl_root: PathLike,
) -> None:
    """deploy grapl to aws"""
    click.echo("deploying grapl cdk stacks to aws")
    aws_cdk_ops.deploy_grapl(
        grapl_root=pathlib.Path(grapl_root).absolute(),
        aws_profile=graplctl_state.aws_profile,
        stdout=click.get_binary_stream("stdout"),
        stderr=click.get_binary_stream("stderr"),
    )
    click.echo("deployed grapl cdk stacks to aws")

    click.echo("creating dgraph cluster in aws")
    if not dgraph_ops.create_dgraph(
        graplctl_state=graplctl_state, instance_type=dgraph_instance_type
    ):
        click.echo("dgraph cluster already exists")
        return
    click.echo("created dgraph cluster in aws")


@aws.command()
@click.option(
    "-a", "--all", is_flag=True, required=True, help="all services and resources"
)
@click.argument(
    "grapl_root",
    type=click.Path(exists=True, file_okay=False, resolve_path=True),
    required=True,
    default=find_grapl_root(),
)
@click.confirmation_option(
    prompt=f"this will tear down the entire grapl deployment, ok?"
)
@pass_graplctl_state
def destroy(graplctl_state: State, all: bool, grapl_root: PathLike) -> None:
    """tear down grapl in aws"""
    click.echo("destroying all grapl aws resources")

    for swarm_id in docker_swarm_ops.swarm_ids(
        ec2=graplctl_state.ec2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
    ):
        click.echo(f"removing swarm dns records for swarm {swarm_id}")
        dgraph_ops.remove_dgraph_dns(graplctl_state=graplctl_state, swarm_id=swarm_id)
        click.echo(f"removed swarm dns records for swarm {swarm_id}")

        click.echo(f"destroying swarm cluster {swarm_id}")
        docker_swarm_ops.destroy_swarm(graplctl_state=graplctl_state, swarm_id=swarm_id)
        click.echo(f"destroyed swarm cluster {swarm_id}")

    click.echo("destroying grapl cdk aws stacks")
    aws_cdk_ops.destroy_grapl(
        grapl_root=pathlib.Path(grapl_root).absolute(),
        aws_profile=graplctl_state.aws_profile,
        stdout=click.get_binary_stream("stdout"),
        stderr=click.get_binary_stream("stderr"),
    )
    click.echo("destroyed grapl cdk aws stacks")

    click.echo("destroyed all grapl aws resources")


@aws.command()
@click.confirmation_option(prompt=f"this will incur aws charges, ok?")
@pass_graplctl_state
def provision(graplctl_state: State) -> None:
    """provision the grapl deployment"""
    click.echo("provisioning grapl deployment")
    aws_cdk_ops.provision_grapl(
        lambda_=graplctl_state.lambda_,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    click.echo("provisioned grapl deployment")
