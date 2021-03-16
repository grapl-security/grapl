from __future__ import annotations

import boto3
import click
from graplctl import __version__, common
from graplctl.common import GraplctlState
from graplctl.queues.commands import queues
from graplctl.dgraph.commands import dgraph
from graplctl.aws.commands import aws
from graplctl.swarm.commands import swarm
import graplctl.swarm.lib as docker_swarm_ops

Tag = common.Tag
Ec2Instance = common.Ec2Instance

#
# main entrypoint for grapctl
#


@click.group()
@click.version_option(version=__version__)
@click.option(
    "-r",
    "--grapl-region",
    type=click.Choice(docker_swarm_ops.REGION_TO_AMI_ID.keys()),
    envvar="GRAPL_REGION",
    help="grapl region to target [$GRAPL_REGION]",
    required=True,
)
@click.option(
    "-n",
    "--grapl-deployment-name",
    type=click.STRING,
    envvar="GRAPL_DEPLOYMENT_NAME",
    help="grapl deployment name [$GRAPL_DEPLOYMENT_NAME]",
    required=True,
)
@click.option(
    "-g",
    "--grapl-version",
    type=click.STRING,
    envvar="GRAPL_VERSION",
    help="grapl version [$GRAPL_VERSION]",
    required=True,
)
@click.option(
    "-p",
    "--aws-profile",
    type=click.STRING,
    envvar="AWS_PROFILE",
    help="aws auth profile [$AWS_PROFILE] (\"default\")",
    default="default",
)
@click.pass_context
def main(
    ctx: click.Context,
    grapl_region: str,
    grapl_deployment_name: str,
    grapl_version: str,
    aws_profile: str,
) -> None:
    session = boto3.Session(profile_name=aws_profile)
    ctx.obj = GraplctlState(
        grapl_region,
        grapl_deployment_name,
        grapl_version,
        aws_profile,
        ec2=session.resource("ec2", region_name=grapl_region),
        ssm=session.client("ssm", region_name=grapl_region),
        cloudwatch=session.client("cloudwatch", region_name=grapl_region),
        sns=session.client("sns", region_name=grapl_region),
        route53=session.client("route53", region_name=grapl_region),
        sqs=session.client("sqs", region_name=grapl_region),
        lambda_=session.client("lambda", region_name=grapl_region),
    )


main.add_command(aws)
main.add_command(dgraph)
main.add_command(queues)
main.add_command(swarm)
