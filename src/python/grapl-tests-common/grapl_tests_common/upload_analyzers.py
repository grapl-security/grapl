from __future__ import annotations

import logging
from typing import TYPE_CHECKING, NamedTuple, Sequence

if TYPE_CHECKING:
    from mypy_boto3_s3 import S3Client


AnalyzerUpload = NamedTuple(
    "AnalyzerUpload",
    (
        ("local_path", str),
        ("s3_key", str),
    ),
)


def upload_analyzers(
    s3_client: S3Client, analyzers: Sequence[AnalyzerUpload], analyzers_bucket: str
) -> None:
    """
    Basically reimplementing upload_local_analyzers.sh
    Janky, since Jesse will have an analyzer-uploader service pretty soon.
    """
    for (local_path, s3_key) in analyzers:
        assert s3_key.startswith("analyzers/"), s3_key
        assert s3_key.endswith("/main.py"), s3_key
        logging.info(f"S3 uploading analyzer from {local_path}")
        with open(local_path, "rb") as f:
            s3_client.put_object(Body=f.read(), Bucket=analyzers_bucket, Key=s3_key)
