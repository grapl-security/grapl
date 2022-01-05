from __future__ import annotations

import boto3
import click
from botocore.client import Config
from grapl_common.env_helpers import (
    CloudWatchClientFactory,
    DynamoDBResourceFactory,
    EC2ResourceFactory,
    Route53ClientFactory,
    S3ClientFactory,
    SNSClientFactory,
    SQSClientFactory,
    SSMClientFactory,
)
from graplctl import __version__, common
from graplctl.aws.commands import aws
from graplctl.common import State
from graplctl.upload.commands import upload

Tag = common.Tag
Ec2Instance = common.Ec2Instance

#
# main entrypoint for grapctl
#

SUPPORTED_REGIONS = list(
    {
        # This list is meant to capture all possible regions to deploy Grapl to.
        # TODO: In the future, replace with `aws ec2 describe-regions`
        "us-east-1",  # Do yourelf a favor and consider a different one
        "us-east-2",
        "us-west-1",
        "us-west-2",
    }
)


@click.group()
@click.version_option(version=__version__)
@click.option(
    "-r",
    "--grapl-region",
    type=click.Choice(SUPPORTED_REGIONS),
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
    schema_table: str,
    schema_properties_table: str,
    dynamic_session_table: str,
) -> None:
    session = boto3.session.Session()
    config = Config(region_name=grapl_region)
    ctx.obj = State(
        grapl_region,
        grapl_deployment_name,
        grapl_version,
        cloudwatch=CloudWatchClientFactory(session).from_env(config=config),
        dynamodb=DynamoDBResourceFactory(session).from_env(config=config),
        ec2=EC2ResourceFactory(session).from_env(config=config),
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
main.add_command(upload)
