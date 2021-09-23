from __future__ import annotations

from os import PathLike
from pathlib import Path

import click
from grapl_tests_common.upload_logs import upload_osquery_logs, upload_sysmon_logs
from graplctl import idempotency_checks
from graplctl.common import State, pass_graplctl_state
from graplctl.upload.lib import upload_analyzer


@click.group()
@click.pass_context
@pass_graplctl_state
def upload(
    graplctl_state: State,
    ctx: click.Context,
) -> None:
    """commands like "upload analyzer" or "upload sysmon logs" """
    # TODO: Disallow any uploads until we've confirmed we've provisioned
    # https://github.com/grapl-security/issue-tracker/issues/340
    assert idempotency_checks.is_grapl_provisioned(
        dynamodb=graplctl_state.dynamodb,
        schema_table=graplctl_state.schema_table,
    ), "You can't upload anything to grapl until it's provisioned."


@upload.command()
@click.option(
    "--analyzer_main_py",
    type=click.Path(exists=True, file_okay=True, dir_okay=False, resolve_path=True),
    required=True,
    help="Path to the analyzer's `main.py`",
)
@click.option(
    "--analyzers-bucket",
    help="Name of the S3 bucket to upload analyzers to",
    type=click.STRING,
    required=True,
    envvar="GRAPL_ANALYZERS_BUCKET",
)
@pass_graplctl_state
def analyzer(
    graplctl_state: State, analyzers_bucket: str, analyzer_main_py: PathLike
) -> None:
    """Upload an analyzer to the S3 bucket"""
    upload_analyzer(
        graplctl_state.s3,
        analyzers_bucket=analyzers_bucket,
        analyzer_main_py=Path(analyzer_main_py).resolve(),
    )


@upload.command()
@click.option(
    "--logfile",
    type=click.Path(exists=True, file_okay=True, dir_okay=False, resolve_path=True),
    required=True,
    help="The log file to upload",
)
@click.option(
    "--log-bucket",
    help="The name of the S3 bucket to which Sysmon logs should be uploaded",
    type=click.STRING,
    required=True,
    envvar="GRAPL_SYSMON_LOG_BUCKET",
)
@click.option(
    "--queue-url",
    help="The URL of the SQS queue for Sysmon logs",
    type=click.STRING,
    required=True,
    envvar="GRAPL_SYSMON_GENERATOR_QUEUE",
)
@pass_graplctl_state
def sysmon(
    graplctl_state: State, logfile: PathLike, log_bucket: str, queue_url: str
) -> None:
    """Upload a Sysmon log file to the S3 bucket"""
    upload_sysmon_logs(
        s3_client=graplctl_state.s3,
        sqs_client=graplctl_state.sqs,
        deployment_name=graplctl_state.grapl_deployment_name,
        log_bucket=log_bucket,
        queue_url=queue_url,
        logfile=Path(logfile).resolve(),
    )


@upload.command()
@click.option(
    "--logfile",
    type=click.Path(exists=True, file_okay=True, dir_okay=False, resolve_path=True),
    required=True,
    help="The log file to upload",
)
@click.option(
    "--log-bucket",
    help="The name of the S3 bucket to which OSQuery logs should be uploaded",
    type=click.STRING,
    required=True,
    envvar="GRAPL_OSQUERY_LOG_BUCKET",
)
@click.option(
    "--queue-url",
    help="The URL of the SQS queue for OSQuery logs",
    type=click.STRING,
    required=True,
    envvar="GRAPL_OSQUERY_GENERATOR_QUEUE",
)
@pass_graplctl_state
def osquery(
    graplctl_state: State, logfile: PathLike, log_bucket: str, queue_url: str
) -> None:
    """Upload an OSQuery log file to the S3 bucket"""
    upload_osquery_logs(
        s3_client=graplctl_state.s3,
        sqs_client=graplctl_state.sqs,
        deployment_name=graplctl_state.grapl_deployment_name,
        log_bucket=log_bucket,
        queue_url=queue_url,
        logfile=Path(logfile).resolve(),
    )
