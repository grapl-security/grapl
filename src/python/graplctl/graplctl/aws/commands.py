from __future__ import annotations

import pathlib

import click
import graplctl.aws.lib as aws_cdk_ops
import graplctl.dgraph.lib as dgraph_ops
import graplctl.swarm.lib as docker_swarm_ops
from graplctl.common import State, pass_graplctl_state

#
# aws deployment & provisioning commands
#


@click.group(help="commands for managing grapl aws resources", name="aws")
@click.pass_context
@pass_graplctl_state
def aws(
    graplctl_state: State,
    ctx: click.Context,
):
    pass


@aws.command(help="deploy grapl to aws", name="deploy")
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
)
@click.pass_obj
def aws_deploy(
    graplctl_state: State, all: bool, dgraph_instance_type: str, grapl_root: str
):
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


@aws.command(help="tear down grapl in aws", name="destroy")
@click.option(
    "-a", "--all", is_flag=True, required=True, help="all services and resources"
)
@click.argument(
    "grapl_root",
    type=click.Path(exists=True, file_okay=False, resolve_path=True),
    required=True,
)
@click.pass_obj
def aws_destroy(graplctl_state: State, all: bool, grapl_root: str):
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


@aws.command(help="provision the grapl deployment", name="provision")
@click.pass_obj
def aws_provision(graplctl_state: State):
    click.echo("provisioning grapl deployment")
    aws_cdk_ops.provision_grapl(
        lambda_=graplctl_state.lambda_,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    click.echo("provisioned grapl deployment")
