import click
from typing import List
from graplctl.common import GraplctlState
from mypy_boto3_sqs import SQSClient
from mypy_boto3_sqs.type_defs import SendMessageBatchRequestEntryTypeDef, MessageTypeDef, DeleteMessageBatchRequestEntryTypeDef

DLQ_SUFFIX = "-dead-letter-queue"
RETRY_SUFFIX = "-retry-queue"

def list_dlqs_for_deployment(
    graplctl_state: GraplctlState,
    sqs_client: SQSClient,
) -> List[str]:
    queues = sqs_client.list_queues(
        QueueNamePrefix=graplctl_state.grapl_deployment_name
    )["QueueUrls"]
    queues = [
        q for q in queues if
        q.endswith(DLQ_SUFFIX)
    ]
    return queues

def redrive_from_dlq(
    graplctl_state: GraplctlState,
    sqs_client: SQSClient,
    dlq_queue_url: str,
) -> None:
    assert dlq_queue_url.startswith("http"), "Please pass a Queue URL (try a `graplctl queues ls`)."
    assert dlq_queue_url.endswith(DLQ_SUFFIX), "You're trying to redrive from a non-dead-letteAr-queue."

    retry_queue_url = _retry_queue_for_dlq(dlq_queue_url)

    while True:
        _messages = sqs_client.receive_message(
            QueueUrl=dlq_queue_url,
            MaxNumberOfMessages=10,
        )
        messages = _messages.get("Messages", None)
        if not messages:
            click.echo("No messages left to redrive")
            return
        else:
            click.echo(f"Redriving {len(messages)} messages into:\n  {retry_queue_url}")
        
        sqs_client.send_message_batch(
            QueueUrl=retry_queue_url,
            Entries=[_into_send_message(m) for m in messages],
        )

        sqs_client.delete_message_batch(
            QueueUrl=dlq_queue_url,
            Entries=[_into_delete_message(m) for m in messages],
        )

def _retry_queue_for_dlq(
    queue_url: str
) -> str:
    return queue_url.replace(DLQ_SUFFIX, RETRY_SUFFIX)

def _into_send_message(m: MessageTypeDef) -> SendMessageBatchRequestEntryTypeDef:
    return {
        "Id": m["MessageId"],
        "MessageBody": m["Body"],
    }
        
def _into_delete_message(m: MessageTypeDef) -> DeleteMessageBatchRequestEntryTypeDef:
    return {
        "Id": m["MessageId"],
        "ReceiptHandle": m["ReceiptHandle"],
    }