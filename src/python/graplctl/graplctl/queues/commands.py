from graplctl.common import GraplctlState, pass_graplctl_state
from graplctl.queues.ls import queues_for_deployment
from mypy_boto3_sqs import SQSClient
import dataclasses
import click

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
    sqs_client = graplctl_state.boto3_session.client("sqs", region_name=graplctl_state.grapl_region)
    ctx.obj = QueuesObj(sqs_client)


@queues.command(help="List redrivable queues")
@pass_queues_obj
@pass_graplctl_state
def ls(
    graplctl_state: GraplctlState,
    queues_obj: QueuesObj,
) -> None:
    """List all redrivable queues"""
    queues = queues_for_deployment(graplctl_state, queues_obj.sqs_client)
    click.echo(list(queues))
