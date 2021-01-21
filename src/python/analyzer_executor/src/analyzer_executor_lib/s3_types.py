from typing import Dict, List, NewType, Tuple

from typing_extensions import TypedDict


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

MessageWithReceipt = Tuple[SQSMessageBody, SQSReceiptHandle]
