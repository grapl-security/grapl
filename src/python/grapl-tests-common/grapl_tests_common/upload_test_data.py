from typing_extensions import Protocol
from grapl_tests_common.types import S3ServiceResource
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
        # i hate this lol
        # but it's probably better than mucking with path and importing that module...
        subprocess.run(
            [
                "python3",
                "/home/grapl/etc/local_grapl/bin/upload-sysmon-logs.py",
                "--bucket_prefix",
                BUCKET_PREFIX,
                "--logfile",
                self.path,
                "--use-links",
                "True",
            ]
        )
