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
        deployment_name=graplctl_state.grapl_deployment_name,
    ), "You can't upload anything to grapl until it's provisioned."


@upload.command()
@click.option(
    "--analyzer_main_py",
    type=click.Path(exists=True, file_okay=True, dir_okay=False, resolve_path=True),
    required=True,
    help="Path to the analyzer's `main.py`",
)
@pass_graplctl_state
def analyzer(graplctl_state: State, analyzer_main_py: PathLike) -> None:
    """Upload an analyzer to the S3 bucket"""
    upload_analyzer(
        graplctl_state.s3,
        graplctl_state.grapl_deployment_name,
        analyzer_main_py=Path(analyzer_main_py).resolve(),
    )


@upload.command()
@click.option(
    "--logfile",
    type=click.Path(exists=True, file_okay=True, dir_okay=False, resolve_path=True),
    required=True,
    help="The log file to upload",
)
@pass_graplctl_state
def sysmon(graplctl_state: State, logfile: PathLike) -> None:
    """Upload a Sysmon log file to the S3 bucket"""
    upload_sysmon_logs(
        s3_client=graplctl_state.s3,
        sqs_client=graplctl_state.sqs,
        deployment_name=graplctl_state.grapl_deployment_name,
        logfile=Path(logfile).resolve(),
    )


@upload.command()
@click.option(
    "--logfile",
    type=click.Path(exists=True, file_okay=True, dir_okay=False, resolve_path=True),
    required=True,
    help="The log file to upload",
)
@pass_graplctl_state
def osquery(graplctl_state: State, logfile: PathLike) -> None:
    """Upload an OSQuery log file to the S3 bucket"""
    upload_osquery_logs(
        s3_client=graplctl_state.s3,
        sqs_client=graplctl_state.sqs,
        deployment_name=graplctl_state.grapl_deployment_name,
        logfile=Path(logfile).resolve(),
    )
