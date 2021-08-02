from __future__ import annotations

import click
import graplctl.aws.lib as aws_lib
from graplctl import idempotency_checks
from graplctl.common import State, pass_graplctl_state
from graplctl.dgraph import commands as dgraph_commands

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
@click.confirmation_option(prompt=f"this will incur aws charges, ok?")
@pass_graplctl_state
def provision(graplctl_state: State) -> None:
    """provision the grapl deployment"""
    # TODO: Add a check that dgraph has been created
    assert not idempotency_checks.is_grapl_provisioned(
        dynamodb=graplctl_state.dynamodb,
        schema_table=graplctl_state.schema_table,
    ), "Grapl is already provisioned!"
    click.echo("provisioning grapl deployment")
    aws_lib.provision_grapl(
        lambda_=graplctl_state.lambda_,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    click.echo("provisioned grapl deployment")


@aws.command()
@click.confirmation_option(
    prompt=f"this will wipe your grapl state. realistically, only for devs. that okay?"
)
@click.confirmation_option(
    prompt=f"really wanna make sure, this will wipe your dgraph and dynamodb"
)
@pass_graplctl_state
@click.pass_context
def wipe_state(ctx: click.Context, graplctl_state: State) -> None:
    """Wipe dynamodb"""
    assert idempotency_checks.is_grapl_provisioned(
        dynamodb=graplctl_state.dynamodb,
        schema_table=graplctl_state.schema_table,
    ), "Grapl hasn't been provisioned yet."
    click.echo("Wiping dynamodb")
    aws_lib.wipe_dynamodb(
        dynamodb=graplctl_state.dynamodb,
        deployment_name=graplctl_state.grapl_deployment_name,
        schema_table_name=graplctl_state.schema_table,
    )
    click.echo("Wiped dynamodb")
    # Also destroy dgraph
    ctx.forward(dgraph_commands.destroy)


@aws.command()
@pass_graplctl_state
def test(graplctl_state: State) -> None:
    """run end-to-end tests in aws"""
    click.echo("running end-to-end tests")
    aws_lib.run_e2e_tests(
        lambda_=graplctl_state.lambda_,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    click.echo("ran end-to-end tests")
