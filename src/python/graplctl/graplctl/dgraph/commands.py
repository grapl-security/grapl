import click
import graplctl.dgraph.lib as dgraph_ops
from graplctl.common import State, pass_graplctl_state

#
# dgraph operational commands
#


@click.group()
@click.pass_context
@pass_graplctl_state
def dgraph(
    graplctl_state: State,
    ctx: click.Context,
) -> None:
    """commands for operating dgraph"""
    pass


@dgraph.command()
@click.option(
    "-t",
    "--instance-type",
    type=click.Choice(choices=("i3.large", "i3.xlarge", "i3.2xlarge")),
    help="EC2 instance type for swarm nodes",
    required=True,
)
@pass_graplctl_state
def create(graplctl_state: State, instance_type: str) -> None:
    """spin up a swarm cluster and deploy dgraph on it"""
    click.echo(f"creating dgraph cluster of {instance_type} instances")
    if not dgraph_ops.create_dgraph(
        graplctl_state=graplctl_state, instance_type=instance_type
    ):
        click.echo("dgraph cluster already exists")
        return
    click.echo(f"created dgraph cluster of {instance_type} instances")


@dgraph.command()
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
def remove_dns(graplctl_state: State, swarm_id: str) -> None:
    """remove dgraph dns records"""
    click.echo(f"removing dgraph dns records for swarm {swarm_id}")
    dgraph_ops.remove_dgraph_dns(graplctl_state=graplctl_state, swarm_id=swarm_id)
    click.echo(f"removed dgraph dns records for swarm {swarm_id}")
