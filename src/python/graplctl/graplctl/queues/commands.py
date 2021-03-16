from __future__ import annotations

import click
from graplctl.common import GraplctlState, pass_graplctl_state
from graplctl.queues.lib import list_dlqs_for_deployment, redrive_from_dlq


@click.group(help="Manipulate work queues")
@click.pass_context
@pass_graplctl_state
def queues(
    graplctl_state: GraplctlState,
    ctx: click.Context,
) -> None:
    pass


@queues.command(help="List redrivable queues")
@pass_graplctl_state
def ls(
    graplctl_state: GraplctlState,
) -> None:
    queues = list_dlqs_for_deployment(graplctl_state, graplctl_state.sqs)
    click.echo("\n".join(queues))


@queues.command(help="Redrive messages from a dead letter queue")
@pass_graplctl_state
@click.argument(
    "dlq_url",
)
def redrive(
    graplctl_state: GraplctlState,
    dlq_url: str,
) -> None:
    redrive_from_dlq(
        graplctl_state,
        graplctl_state.sqs,
        dlq_url,
    )
