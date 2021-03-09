from typing import List
from graplctl.common import GraplctlState
from mypy_boto3_sqs import SQSClient

def queues_for_deployment(
    graplctl_state: GraplctlState,
    sqs_client: SQSClient,
) -> List[str]:
    queues = sqs_client.list_queues(
        QueueNamePrefix=graplctl_state.grapl_deployment_name
    )["QueueUrls"]
    return queues

def partition_queues(
    all_queues: List[str]
) -> 