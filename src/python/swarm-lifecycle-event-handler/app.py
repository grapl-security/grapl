import json
import logging
import os
import sys

from typing import Dict, Optional

import boto3

from chalice import Chalice

app = Chalice(app_name="swarm-lifecycle-event-handler")

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "ERROR"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

INSTANCE_LAUNCHING = "autoscaling:EC2_INSTANCE_LAUNCHING"
INSTANCE_TERMINATING = "autoscaling:EC2_INSTANCE_TERMINATING"


def _instance_ip_address(instance_id: str) -> str:
    ec2 = boto3.resource("ec2")
    instance = ec2.Instance(instance_id)
    return instance.private_ip_address


def _complete_lifecycle_action(
    lifecycle_hook_name: str, autoscaling_group_name: str, instance_id: str
) -> None:
    autoscaling = boto3.resource("autoscaling")
    response = autoscaling.complete_lifecycle_action(
        LifecycleHookName=lifecycle_hook_name,
        AutoScalingGroupName=autoscaling_group_name,
        InstanceId=instance_id,
        LifecycleActionResult="CONTINUE",
    )
    LOGGER.info(f"{json.dumps(response)}")


def _remove_dns_ip(dns_name: str, ip_address: str, hosted_zone_id: str) -> None:
    route53 = boto3.resource("route53")
    response = route53.change_resource_record_sets(
        HostedZoneId=hosted_zone_id,
        ChangeBatch={
            "Changes": [
                {
                    "Action": "DELETE",
                    "ResourceRecordSet": {
                        "Name": dns_name,
                        "Type": "A",
                        "TTL": 300,
                        "ResourceRecords": [
                            {"Value": ip_address},
                        ],
                    },
                },
            ],
            "Comment": f"Remove {ip_address} from A Record for {dns_name}",
        },
    )
    LOGGER.info(f"{json.dumps(response)}")


def _insert_dns_ip(dns_name: str, ip_address: str, hosted_zone_id: str) -> None:
    route53 = boto3.resource("route53")
    response = route53.change_resource_record_sets(
        HostedZoneId=hosted_zone_id,
        ChangeBatch={
            "Changes": [
                {
                    "Action": "UPSERT",
                    "ResourceRecordSet": {
                        "Name": dns_name,
                        "Type": "A",
                        "TTL": 300,
                        "ResourceRecords": [
                            {"Value": ip_address},
                        ],
                    },
                },
            ],
            "Comment": f"Insert {ip_address} into A Record for {dns_name}",
        },
    )
    LOGGER.info(f"{json.dumps(response)}")


@app.lambda_function()
def main(event, context) -> None:
    for record in event["Records"]:
        if "Event" in record:
            transition = record["Event"]

            if "NotificationMetadata" in record:
                notification_metadata = json.loads(record["NotificationMetadata"])
                hosted_zone_id = notification_metadata["HostedZoneId"]
                dns_name = notification_metadata["DnsName"]
                prefix = notification_metadata["Prefix"]
            else:
                LOGGER.warn(
                    "NotificationMetadata absent from record, skipping: {record}"
                )
                continue

            if "EC2InstanceId" in record:
                instance_id = record["EC2InstanceId"]
                ip_address = _instance_ip_address(instance_id)
            else:
                LOGGER.warm("EC2InstanceId absent from record, skipping: {record}")
                continue

            try:
                if transition == INSTANCE_LAUNCHING:
                    try:
                        _insert_dns_ip(dns_name, ip_address, hosed_zone_id)
                    finally:
                        _complete_lifecycle_action(
                            lifecycle_hook_name=f"{prefix}-SwarmLaunchHook",
                            autoscaling_group_name=autoscaling_group_name,
                            instance_id=instance_id
                        )
                elif transition == INSTANCE_TERMINATING:
                    try:
                        _remove_dns_ip(dns_name, ip_address, hosted_zone_id)
                    finally:
                        _complete_lifecycle_action(
                            lifecycle_hook_name=f"{prefix}-SwarmTerminateHook",
                            autoscaling_group_name=autoscaling_group_name,
                            instance_id=instance_id
                        )
                else:
                    LOGGER.warn(
                        f'Encountered unknown lifecycle transition "{transition}" in record: {record}'
                    )
        else:
            LOGGER.warn("Encountered unknown record: {record}")
