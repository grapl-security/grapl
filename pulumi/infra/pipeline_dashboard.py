"""
This is an overall hacky port of what CDK produced.
Feel free to replace the SQS metrics with Kafka queue metrics asap.
Feel free to replace all of this with a Grafana dashboard asap.
"""

import json
from typing import Any, Dict, List, Mapping, Optional, Sequence, Tuple, Union
from pulumi.output import Output

import pulumi_aws as aws
import pulumi_docker as docker
from infra.cache import Cache
from infra.config import DEPLOYMENT_NAME, SERVICE_LOG_RETENTION_DAYS
from infra.ec2 import Ec2Port
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService
from infra.lambda_ import Lambda
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.policies import ECR_TOKEN_POLICY, attach_policy
from infra.repository import Repository, registry_credentials
from infra.service_queue import ServiceQueue
from pulumi_aws.cloudwatch import dashboard
from typing_extensions import Literal

import pulumi


def service_queue_widget(queues: ServiceQueue) -> Dict[str, Any]:
    properties = {
        "view": "timeSeries",
        "title": f"Queues for {queues._name}",
        "region": "us-east-1",
        "metrics": [
            [
                "AWS/SQS",
                "NumberOfMessagesReceived",
                "QueueName",
                queues.queue.name,
                {"color": "#2ca02c", "label": "Queue", "stat": "Sum"},
            ],
            [
                "AWS/SQS",
                "NumberOfMessagesReceived",
                "QueueName",
                queues.retry_queue.name,
                {"color": "#ff7f0e", "label": "Retry", "stat": "Sum"},
            ],
            [
                "AWS/SQS",
                "ApproximateNumberOfMessagesVisible",
                "QueueName",
                queues.dead_letter_queue.name,
                {"color": "#d62728", "label": "Dead", "stat": "Maximum"},
            ],
        ],
        "yAxis": {},
        "liveData": True,
    }

    return {
        "type": "metric",
        "width": 24,
        "height": 6,
        "x": 0,
        "y": 6,
        "properties": properties,
    }


"""
def lambda_invoke_widget(lambda_resource: Lambda) -> Dict[str, Any]:
    assert isinstance(lambda_resource.function._name, str)

    properties = {
        "view": "timeSeries",
        "title": f"Invoke {lambda_resource.function._name}",
        "region": "us-east-1",
        "metrics": [
            [
                "AWS/Lambda",
                "Invocations",
                "FunctionName",
                lambda_resource.function._name,
                {"color": "#1f77b4", "stat": "Sum"},
            ],
            [
                "AWS/Lambda",
                "Errors",
                "FunctionName",
                lambda_resource.function._name,
                {"color": "#d62728", "stat": "Sum"},
            ],
        ],
        "yAxis": {},
        "liveData": True,
    }
    return {
        "type": "metric",
        "width": 12,
        "height": 6,
        "x": 0,
        "y": 36,
        "properties": properties,
    }
"""

class PipelineDashboard(pulumi.ComponentResource):
    def __init__(
        self, 
        fargate_services: List[FargateService],
        lambdas: List[Lambda],
    ) -> None:

        def into_dashboard_body(args: Dict[str, Any]) -> str:
            fargate_services = args['fargate_services']
            return json.dumps({
                "widgets": [
                    service_queue_widget(fg_svc.queue) for fg_svc in fargate_services
                ] 
                # + [ lambda_invoke_widget(lambda_resource) for lambda_resource in lambdas]
            })
        dashboard_body = Output.all(
            fargate_services=fargate_services
        ).apply(into_dashboard_body)

        dashboard = aws.cloudwatch.Dashboard(
            "pipeline-dashboard",
            dashboard_body=dashboard_body,
            dashboard_name=f"{DEPLOYMENT_NAME}-pipeline-dashboard",
            opts=pulumi.ResourceOptions(),
        )
