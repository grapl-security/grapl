from __future__ import annotations

import json
import logging
import sys
import time

from typing import TYPE_CHECKING, Any, Iterator, List, NamedTuple, Optional

import boto3

if TYPE_CHECKING:
    from mypy_boto3_ec2 import EC2ServiceResource
    from mypy_boto3_cloudwatch.client import CloudWatchClient
    from mypy_boto3_cloudwatch.type_defs import MetricTypeDef
    from mypy_boto3_sns.client import SNSClient


CW_NAMESPACE = "CWAgent"
CW_DISK_USAGE_METRIC_NAME = "disk_used_percent"


def _find_operational_alarms_arn(
    prefix: str,
    sns: Optional[SNSClient] = None,
) -> str:
    sns = sns or boto3.client("sns")
    topics_raw = sns.list_topics()
    all_topic_arns = [d["TopicArn"] for d in topics_raw["Topics"]]

    def seems_like_the_desired_arn(arn: str) -> bool:
        # see CDK class OperationalAlarms
        # note: the prefix should *not* be lower-ized
        return f"{prefix}-operational-alarms-sink" in arn

    arn = next((arn for arn in all_topic_arns if seems_like_the_desired_arn(arn)), None)
    if not arn:
        raise Exception(f"Couldn't find a good candidate arn among {all_topic_arns}")
    return arn


def _find_metric_for_instance(
    cloudwatch: CloudWatchClient,
    instance_id: str,
    path: str,
) -> MetricTypeDef:
    """
    To define a Cloudwatch Alarm, one must specify *all* the dimensions complete with values.
    (The dimension names are part of the identity of the metric. Gross!)
    So, before we define an alarm, we do a quick query on path + instance id to get the
    other dimensions.
    """
    metrics_result = cloudwatch.list_metrics(
        Namespace=CW_NAMESPACE,
        MetricName=CW_DISK_USAGE_METRIC_NAME,
        Dimensions=[
            {
                "Name": "path",
                "Value": path,
            },
            {
                "Name": "InstanceId",
                "Value": instance_id,
            },
            {
                "Name": "AutoScalingGroupName",
            },
            {
                "Name": "ImageId",
            },
            {
                "Name": "InstanceType",
            },
            {
                "Name": "device",
            },
            {"Name": "fstype"},
        ],
    )
    metrics = metrics_result["Metrics"]
    if len(metrics) != 1:
        raise Exception(
            f"Tried querying for disk metrics in path {path} on {instance_id} - expected 1, got {metrics}\n"
            "(Try waiting ~5m after provisioning an Autoscaling Group for the expected metric to show up.)"
        )
    return metrics[0]


def _create_disk_usage_alarms(
    cloudwatch: CloudWatchClient,
    instance_id: str,
    prefix: str,
) -> None:
    ops_alarm_action = _find_operational_alarms_arn(prefix)

    root_metric = _find_metric_for_instance(cloudwatch, instance_id, path="/")
    """Create disk usage alarms for the / and /dgraph partitions"""
    cloudwatch.put_metric_alarm(
        AlarmActions=[ops_alarm_action],
        AlarmName=f"/ disk_used_percent ({instance_id})",
        AlarmDescription=f"Root volume disk usage percent threshold exceeded on {instance_id}",
        ActionsEnabled=False,
        MetricName=root_metric["MetricName"],
        Namespace=root_metric["Namespace"],
        Statistic="Maximum",
        Period=300,
        EvaluationPeriods=1,
        ComparisonOperator="GreaterThanOrEqualToThreshold",
        Threshold=95.0,
        Unit="Percent",
        Dimensions=root_metric["Dimensions"],
    )

    dgraph_partition_metric = _find_metric_for_instance(
        cloudwatch, instance_id, path="/dgraph"
    )
    cloudwatch.put_metric_alarm(
        AlarmActions=[ops_alarm_action],
        AlarmName=f"/dgraph disk_used_percent ({instance_id})",
        AlarmDescription=f"DGraph volume disk usage percent threshold exceeded on {instance_id}",
        ActionsEnabled=False,
        MetricName=dgraph_partition_metric["MetricName"],
        Namespace=dgraph_partition_metric["Namespace"],
        Statistic="Maximum",
        Period=300,
        EvaluationPeriods=1,
        ComparisonOperator="GreaterThanOrEqualToThreshold",
        Threshold=95.0,
        Unit="Percent",
        Dimensions=dgraph_partition_metric["Dimensions"],
    )
