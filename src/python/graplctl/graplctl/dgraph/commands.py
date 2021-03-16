from __future__ import annotations

import click

from graplctl.common import GraplctlState, pass_graplctl_state
import graplctl.dgraph.lib as dgraph_ops


#
# dgraph operational commands
#


@click.group(help="commands for operating dgraph", name="dgraph")
@click.pass_context
@pass_graplctl_state
def dgraph(
    graplctl_state: GraplctlState,
    ctx: click.Context,
):
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
@pass_graplctl_state
def create_dgraph(graplctl_state: GraplctlState, instance_type: str):
    click.echo(f"creating dgraph cluster of {instance_type} instances")
    if not dgraph_ops.create_dgraph(graplctl_state=graplctl_state, instance_type=instance_type):
        click.echo("dgraph cluster already exists")
        return
    click.echo(f"created dgraph cluster of {instance_type} instances")


@dgraph.command(help="remove dgraph dns records", name="remove-dns")
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique id of the swarm cluster",
    required=True,
)
@click.confirmation_option(
    prompt="are you sure you want to remove the dgraph dns records?"
)
@pass_graplctl_state
def dgraph_remove_dns(graplctl_state: GraplctlState, swarm_id: str):
    click.echo(f"removing dgraph dns records for swarm {swarm_id}")
    dgraph_ops.remove_dgraph_dns(graplctl_state=graplctl_state, swarm_id=swarm_id)
    click.echo(f"removed dgraph dns records for swarm {swarm_id}")
