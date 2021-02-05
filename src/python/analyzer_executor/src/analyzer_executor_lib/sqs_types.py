from __future__ import annotations

import json
from typing import TYPE_CHECKING, Dict, List, NewType

from typing_extensions import TypedDict

if TYPE_CHECKING:
    from mypy_boto3_sqs.type_defs import MessageTypeDef


class S3BucketDict(TypedDict):
    name: str
    ownerIdentity: Dict
    arn: str


class S3ObjectDict(TypedDict):
    key: str
    size: str
    versionId: str


class S3DescriptorDict(TypedDict):
    """
    The s3 key provides information about the bucket and
    object involved in the event. The object key name value
    is URL encoded.
    """

    bucket: Dict
    object: Dict


class S3PutRecordDict(TypedDict):
    # basically, s3:ObjectCreated:Put
    awsRegion: str
    eventName: str
    eventSource: str
    eventTime: str
    s3: S3DescriptorDict


class SQSMessageBody(TypedDict):
    Records: List[S3PutRecordDict]


SQSReceiptHandle = NewType("SQSReceiptHandle", str)
SQSMessageId = NewType("SQSMessageId", str)


class SQSMessage:
    """
    Apply some more hardened types to MessageTypeDef, and do the json-load on user's behalf.
    """

    def __init__(self, msg: MessageTypeDef) -> None:
        self.body: SQSMessageBody = json.loads(msg["Body"])
        self.receipt_handle = SQSReceiptHandle(msg["ReceiptHandle"])
        self.message_id = SQSMessageId(msg["MessageId"])
