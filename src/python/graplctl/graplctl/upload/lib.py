from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import click
from grapl_tests_common.upload_analyzers import AnalyzerUpload, upload_analyzers

if TYPE_CHECKING:
    from mypy_boto3_s3 import S3Client


def upload_analyzer(
    s3_client: S3Client,
    analyzers_bucket: str,
    analyzer_main_py: Path,
) -> None:
    # The assumption is that the input path is something like
    # /home/grapl/etc/local_grapl/unique_cmd_parent/main.py
    # where the 'unique_cmd_parent' is the name of the analyzer
    name = analyzer_main_py.parent.name
    s3_key = f"analyzers/{name}/main.py"
    upload_request = AnalyzerUpload(local_path=str(analyzer_main_py), s3_key=s3_key)
    upload_analyzers(
        s3_client=s3_client,
        analyzers=(upload_request,),
        analyzers_bucket=analyzers_bucket,
    )
    click.echo(f"Uploaded analyzer '{name}'")
