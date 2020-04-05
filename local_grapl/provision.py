import boto3
import json

sqs = boto3.client('sqs', endpoint_url="http://localhost:9324")
s3 = boto3.client(
    's3',
    endpoint_url="http://localhost:9000",
    aws_access_key_id='minioadmin',
    aws_secret_access_key='minioadmin',
)

BUCKET_PREFIX = 'local-grapl'

services = (
    'sysmon-graph-generator',
    'generic-graph-generator',
    'node-identifier',
    'graph-merger',
    'analyzer-dispatcher',
    'analyzer-executor',
    'engagement-creator',
)

buckets = (
    BUCKET_PREFIX + '-sysmon-log-bucket',
    BUCKET_PREFIX + '-generic-raw-log-bucket',
    BUCKET_PREFIX + '-unid-subgraphs-generated-bucket',
    BUCKET_PREFIX + '-subgraphs-generated-bucket',
    BUCKET_PREFIX + '-subgraphs-merged-bucket',
    BUCKET_PREFIX + '-analyzer-dispatched-bucket',
    BUCKET_PREFIX + '-analyzers-bucket',
    BUCKET_PREFIX + '-analyzer-matched-subgraphs-bucket',
)

def provision_sqs(service_name: str):
    redrive_queue = sqs.create_queue(
        QueueName=service_name + '-retry-queue',
        Attributes={
            'MessageRetentionPeriod': '86400'
        }

    )

    redrive_url = redrive_queue['QueueUrl']
    redrive_arn = sqs.get_queue_attributes(
        QueueUrl=redrive_url,
        AttributeNames=['QueueArn']
    )['Attributes']['QueueArn']

    redrive_policy = {
        'deadLetterTargetArn': redrive_arn,
        'maxReceiveCount': '10',
    }

    queue = sqs.create_queue(
        QueueName=service_name + '-queue',
    )

    sqs.set_queue_attributes(
        QueueUrl=queue['QueueUrl'],
        Attributes={
            'RedrivePolicy': json.dumps(redrive_policy)
        }
    )
    print(queue['QueueUrl'])

    sqs.purge_queue(QueueUrl=queue['QueueUrl'])
    sqs.purge_queue(QueueUrl=redrive_queue['QueueUrl'])


def provision_bucket(bucket_name: str):
    try:
        s3.create_bucket(Bucket=bucket_name)
    except Exception as e:
        pass
    print(bucket_name)


if __name__ == '__main__':

    for service in services:
        provision_sqs(service)

    for bucket_name in buckets:
        provision_bucket(bucket_name)
