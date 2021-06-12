from __future__ import annotations

import click
import graplctl.aws.lib as aws_cdk_ops
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
    aws_cdk_ops.provision_grapl(
        lambda_=graplctl_state.lambda_,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    click.echo("provisioned grapl deployment")


@aws.command()
@pass_graplctl_state
def test(graplctl_state: State) -> None:
    """run end-to-end tests in aws"""
    click.echo("running end-to-end tests")
    aws_cdk_ops.run_e2e_tests(
        lambda_=graplctl_state.lambda_,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    click.echo("ran end-to-end tests")
