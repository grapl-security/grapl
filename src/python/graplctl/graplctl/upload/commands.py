from __future__ import annotations

from os import PathLike
from pathlib import Path

import click
from graplctl.common import State, pass_graplctl_state
from graplctl.upload.lib import upload_analyzer


@click.group()
@click.pass_context
@pass_graplctl_state
def upload(
    graplctl_state: State,
    ctx: click.Context,
):
    """commands like "upload analyzer" or "upload sysmon logs" """
    pass


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
        Path(analyzer_main_py).resolve(),
        graplctl_state.grapl_deployment_name,
    )
