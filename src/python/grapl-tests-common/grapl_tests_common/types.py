from typing import Any, NamedTuple

# mypy later maybe
S3ServiceResource = Any
SqsServiceResource = Any

AnalyzerUpload = NamedTuple(
    "AnalyzerUpload",
    (
        ("local_path", str),
        ("s3_key", str),
    ),
)
