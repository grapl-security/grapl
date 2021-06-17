from __future__ import annotations

import click
import graplctl.aws.lib as aws_lib
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
@click.confirmation_option(prompt=f"this will incur aws charges, ok?")
@pass_graplctl_state
def provision(graplctl_state: State) -> None:
    """provision the grapl deployment"""
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
def wipe_dynamodb(graplctl_state: State) -> None:
    """Wipe dynamodb"""
    click.echo("Wiping dynamodb")
    aws_lib.wipe_dynamodb(
        dynamodb=graplctl_state.dynamodb,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    click.echo("don't forget to graplctl dgraph destroy + create")


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
