from __future__ import annotations

import boto3
import click
import graplctl.swarm.lib as docker_swarm_ops
from botocore.client import Config
from grapl_common.env_helpers import (
    CloudWatchClientFactory,
    DynamoDBResourceFactory,
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
    type=click.Choice(list(docker_swarm_ops.REGION_TO_AMI_ID.keys())),
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
@click.option(
    "--schema-table",
    type=click.STRING,
    envvar="GRAPL_SCHEMA_TABLE",
    help="The name of the DynamoDB table that holds the schema",
)
@click.option(
    "--schema-properties-table",
    type=click.STRING,
    envvar="GRAPL_SCHEMA_PROPERTIES_TABLE",
    help="The name of the DynamoDB table that holds the schema properties",
)
@click.option(
    "--dynamic-session-table",
    type=click.STRING,
    envvar="GRAPL_DYNAMIC_SESSION_TABLE",
    help="The name of the DynamoDB table that holds dynamic session information",
)
@click.pass_context
def main(
    ctx: click.Context,
    grapl_region: str,
    grapl_deployment_name: str,
    grapl_version: str,
    aws_profile: str,
    schema_table: str,
    schema_properties_table: str,
    dynamic_session_table: str,
) -> None:
    session = boto3.session.Session(profile_name=aws_profile)
    config = Config(region_name=grapl_region)
    lambda_config = Config(
        read_timeout=60 * 15 + 10  # 10s longer than e2e-test-runner and provisioner
    ).merge(config)
    ctx.obj = State(
        grapl_region,
        grapl_deployment_name,
        grapl_version,
        aws_profile,
        cloudwatch=CloudWatchClientFactory(session).from_env(config=config),
        dynamodb=DynamoDBResourceFactory(session).from_env(config=config),
        ec2=EC2ResourceFactory(session).from_env(config=config),
        lambda_=LambdaClientFactory(session).from_env(config=lambda_config),
        route53=Route53ClientFactory(session).from_env(config=config),
        s3=S3ClientFactory(session).from_env(config=config),
        sns=SNSClientFactory(session).from_env(config=config),
        sqs=SQSClientFactory(session).from_env(config=config),
        ssm=SSMClientFactory(session).from_env(config=config),
        schema_table=schema_table,
        schema_properties_table=schema_properties_table,
        dynamic_session_table=dynamic_session_table,
    )


main.add_command(aws)
main.add_command(dgraph)
main.add_command(queues)
main.add_command(swarm)
main.add_command(upload)
