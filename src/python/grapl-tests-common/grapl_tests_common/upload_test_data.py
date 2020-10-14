from typing_extensions import Protocol
from grapl_tests_common.types import S3ServiceResource
from grapl_tests_common.upload_sysmon_logs import upload_sysmon_logs
import logging
import subprocess

BUCKET_PREFIX = "local-grapl"


class UploadTestData(Protocol):
    def upload(self, s3_client: S3ServiceResource) -> None:
        pass


class UploadSysmonLogsTestData(UploadTestData):
    def __init__(self, path: str) -> None:
        self.path = path

    def upload(self, s3_client: S3ServiceResource) -> None:
        logging.info(f"S3 uploading test data from {self.path}")
        upload_sysmon_logs(
            prefix=BUCKET_PREFIX,
            logfile=self.path,
            use_links=True,
        )
