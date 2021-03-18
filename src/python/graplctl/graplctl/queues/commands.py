from __future__ import annotations

import click
from graplctl.common import State, pass_graplctl_state
from graplctl.queues.lib import list_dlqs_for_deployment, redrive_from_dlq


@click.group()
@click.pass_context
@pass_graplctl_state
def queues(
    graplctl_state: State,
    ctx: click.Context,
) -> None:
    """manipulate work queues"""
    pass


@queues.command()
@pass_graplctl_state
def ls(
    graplctl_state: State,
) -> None:
    """list redrivable queues"""
    queues = list_dlqs_for_deployment(graplctl_state, graplctl_state.sqs)
    click.echo("\n".join(queues))


@queues.command()
@pass_graplctl_state
@click.argument(
    "dlq_url",
)
def redrive(
    graplctl_state: State,
    dlq_url: str,
) -> None:
    """redrive messages from a dead letter queue"""
    redrive_from_dlq(
        graplctl_state,
        graplctl_state.sqs,
        dlq_url,
    )
