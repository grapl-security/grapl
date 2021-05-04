from __future__ import annotations

import boto3
import click
import graplctl.swarm.lib as docker_swarm_ops
from grapl_common.env_helpers import (
    CloudWatchClientFactory,
    EC2ResourceFactory,
    LambdaClientFactory,
    Route53ClientFactory,
    S3ClientFactory,
    SNSClientFactory,
    SQSClientFactory,
    SSMClientFactory,
)
from graplctl import __version__, common
from graplctl.aws.commands import aws
from graplctl.common import State
from graplctl.dgraph.commands import dgraph
from graplctl.queues.commands import queues
from graplctl.swarm.commands import swarm
from graplctl.upload.commands import upload

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
    envvar="DEPLOYMENT_NAME",
    help="grapl deployment name [$DEPLOYMENT_NAME]",
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
    help='aws auth profile [$AWS_PROFILE] ("default")',
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
    ctx.obj = State(
        grapl_region,
        grapl_deployment_name,
        grapl_version,
        aws_profile,
        ec2=EC2ResourceFactory(session).from_env(region=grapl_region),
        ssm=SSMClientFactory(session).from_env(region=grapl_region),
        cloudwatch=CloudWatchClientFactory(session).from_env(region=grapl_region),
        s3=S3ClientFactory(session).from_env(region=grapl_region),
        sns=SNSClientFactory(session).from_env(region=grapl_region),
        route53=Route53ClientFactory(session).from_env(region=grapl_region),
        sqs=SQSClientFactory(session).from_env(region=grapl_region),
        lambda_=LambdaClientFactory(session).from_env(region=grapl_region),
    )


main.add_command(aws)
main.add_command(dgraph)
main.add_command(queues)
main.add_command(swarm)
main.add_command(upload)
