from __future__ import annotations

import dataclasses
from typing import TYPE_CHECKING

import click
from graplctl.common import GraplctlState, pass_graplctl_state
from graplctl.queues.lib import list_dlqs_for_deployment, redrive_from_dlq

if TYPE_CHECKING:
    from mypy_boto3_sqs import SQSClient


@dataclasses.dataclass
class QueuesObj:
    sqs_client: SQSClient


pass_queues_obj = click.make_pass_decorator(QueuesObj)


@click.group(help="Manipulate work queues")
@click.pass_context
@pass_graplctl_state
def queues(
    graplctl_state: GraplctlState,
    ctx: click.Context,
) -> None:
    sqs_client = graplctl_state.boto3_session.client(
        "sqs", region_name=graplctl_state.grapl_region
    )
    ctx.obj = QueuesObj(sqs_client)


@queues.command(help="List redrivable queues")
@pass_queues_obj
@pass_graplctl_state
def ls(
    graplctl_state: GraplctlState,
    queues_obj: QueuesObj,
) -> None:
    queues = list_dlqs_for_deployment(graplctl_state, queues_obj.sqs_client)
    click.echo("\n".join(queues))


@queues.command(help="Redrive messages from a dead letter queue")
@pass_queues_obj
@pass_graplctl_state
@click.argument(
    "dlq_url",
)
def redrive(
    graplctl_state: GraplctlState,
    queues_obj: QueuesObj,
    dlq_url: str,
) -> None:
    redrive_from_dlq(
        graplctl_state,
        queues_obj.sqs_client,
        dlq_url,
    )
