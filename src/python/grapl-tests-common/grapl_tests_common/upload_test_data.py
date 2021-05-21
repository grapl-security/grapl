from __future__ import annotations

import logging
from os import PathLike
from typing import TYPE_CHECKING

from grapl_common.env_helpers import get_deployment_name
from grapl_tests_common.upload_logs import upload_sysmon_logs
from typing_extensions import Protocol

if TYPE_CHECKING:
    from mypy_boto3_s3 import S3Client
    from mypy_boto3_sqs import SQSClient


class UploadTestData(Protocol):
    def upload(self, s3_client: S3Client, sqs_client: SQSClient) -> None:
        pass


class UploadSysmonLogsTestData(UploadTestData):
    def __init__(self, path: PathLike) -> None:
        self.path = path

    def upload(self, s3_client: S3Client, sqs_client: SQSClient) -> None:
        logging.info(f"S3 uploading test data from {self.path}")
        upload_sysmon_logs(
            deployment_name=get_deployment_name(),
            logfile=self.path,
            s3_client=s3_client,
            sqs_client=sqs_client,
        )
