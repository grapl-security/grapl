from __future__ import annotations

import click
import graplctl.aws.lib as aws_lib
from graplctl import idempotency_checks
from graplctl.common import State, pass_graplctl_state

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
@click.confirmation_option(
    prompt=f"this will wipe your grapl state. realistically, only for devs. that okay?"
)
@click.confirmation_option(
    prompt=f"really wanna make sure, this will wipe your dynamodb"
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
        schema_table_name=graplctl_state.schema_table,
        schema_properties_table_name=graplctl_state.schema_properties_table,
        dynamic_session_table_name=graplctl_state.dynamic_session_table,
    )
    click.echo("Wiped dynamodb")
