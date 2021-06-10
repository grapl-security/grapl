"""
This is an overall hacky port of what CDK produced.
Feel free to replace the SQS metrics with Kafka queue metrics asap.
Feel free to replace all of this with a Grafana dashboard asap.
"""

import json
from typing import Any, Dict, List

import pulumi_aws as aws
from infra.config import DEPLOYMENT_NAME
from infra.fargate_service import FargateService
from infra.lambda_ import Lambda
from infra.service_queue import ServiceQueueNames

import pulumi
from pulumi.output import Output


def service_queue_widget(names: ServiceQueueNames) -> Dict[str, Any]:
    properties = {
        "view": "timeSeries",
        "title": f"Queues for {names.service_name}",
        "region": "us-east-1",
        "metrics": [
            [
                "AWS/SQS",
                "NumberOfMessagesReceived",
                "QueueName",
                names.queue,
                {"color": "#2ca02c", "label": "Queue", "stat": "Sum"},
            ],
            [
                "AWS/SQS",
                "NumberOfMessagesReceived",
                "QueueName",
                names.retry_queue,
                {"color": "#ff7f0e", "label": "Retry", "stat": "Sum"},
            ],
            [
                "AWS/SQS",
                "ApproximateNumberOfMessagesVisible",
                "QueueName",
                names.dead_letter_queue,
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
        def create_dashboard_json(args: Dict[str, Any]) -> str:
            service_queue_names: List[ServiceQueueNames] = args["service_queue_names"]
            widgets: List[Dict[str, Any]] = [
                service_queue_widget(sqn) for sqn in service_queue_names
            ]
            return json.dumps({"widgets": widgets})

        dashboard_body = Output.all(
            service_queue_names=[fgs.queue.queue_names for fgs in fargate_services]
        ).apply(create_dashboard_json)

        dashboard = aws.cloudwatch.Dashboard(
            "pipeline-dashboard",
            dashboard_body=dashboard_body,
            dashboard_name=f"{DEPLOYMENT_NAME}-pipeline-dashboard",
            opts=pulumi.ResourceOptions(),
        )
