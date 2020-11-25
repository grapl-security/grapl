import json
import logging
import os
import sys
from typing import Any, Dict, Iterator, Optional

import boto3
from botocore.exceptions import ClientError
from chalice import Chalice

app = Chalice(app_name="swarm-lifecycle-event-handler")

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "ERROR"))
if bool(os.environ.get("IS_LOCAL", False)):
    LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

INSTANCE_LAUNCHING = "autoscaling:EC2_INSTANCE_LAUNCHING"
INSTANCE_TERMINATING = "autoscaling:EC2_INSTANCE_TERMINATING"


def _instance_ip_address(instance_id: str) -> str:
    ec2 = boto3.resource("ec2")
    instance = ec2.Instance(instance_id)
    return instance.private_ip_address


def _dns_ip_addresses(
    route53: Any, dns_name: str, ip_address: Optional[str], hosted_zone_id: str
) -> Iterator[str]:
    for rrset in route53.list_resource_record_sets(
        HostedZoneId=hosted_zone_id,
        StartRecordName=dns_name,
    )["ResourceRecordSets"]:
        if rrset["Type"] == "A":
            for rrecord in rrset["ResourceRecords"]:
                yield rrecord["Value"]
    if ip_address is not None:
        yield ip_address


def _complete_lifecycle_action(
    lifecycle_hook_name: str, autoscaling_group_name: str, instance_id: str
) -> None:
    autoscaling = boto3.client("autoscaling")
    autoscaling.complete_lifecycle_action(
        LifecycleHookName=lifecycle_hook_name,
        AutoScalingGroupName=autoscaling_group_name,
        InstanceId=instance_id,
        LifecycleActionResult="CONTINUE",
    )
    LOGGER.info(
        f"Completed {lifecycle_hook_name} lifecycle action for instance {instance_id} in ASG {autoscaling_group_name}"
    )


def _remove_dns_ip(dns_name: str, ip_address: str, hosted_zone_id: str) -> None:
    route53 = boto3.client("route53")
    ip_addresses = [
        ip
        for ip in _dns_ip_addresses(route53, dns_name, None, hosted_zone_id)
        if ip != ip_address
    ]

    change = {
        "Action": "DELETE",  # delete the A record if this is the last address
        "ResourceRecordSet": {
            "Name": dns_name,
            "Type": "A",
            "TTL": 300,
            "ResourceRecords": [{"Value": ip_address}],
        },
    }
    if len(ip_addresses) > 0:
        change["Action"] = "UPSERT"
        change["ResourceRecordSet"]["ResourceRecords"] = [
            {"Value": ip} for ip in ip_addresses
        ]

    try:
        comment = f"Removed {ip_address} from {dns_name} DNS A Record"
        route53.change_resource_record_sets(
            HostedZoneId=hosted_zone_id,
            ChangeBatch={
                "Changes": [change],
                "Comment": comment,
            },
        )
        LOGGER.info(comment)
    except ClientError as e:
        if e.response["Error"]["Code"] == "InvalidChangeBatch":
            LOGGER.warn(f"DNS record does not exist for {ip_address}")
        else:
            raise e


def _insert_dns_ip(dns_name: str, ip_address: str, hosted_zone_id: str) -> None:
    route53 = boto3.client("route53")
    comment = f"Inserted {ip_address} into {dns_name} DNS A Record"
    route53.change_resource_record_sets(
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
                            {"Value": ip}
                            for ip in _dns_ip_addresses(
                                route53, dns_name, ip_address, hosted_zone_id
                            )
                        ],
                    },
                },
            ],
            "Comment": comment,
        },
    )
    LOGGER.info(comment)


@app.lambda_function()
def main(event, context) -> None:
    LOGGER.debug(f"Processing event: {json.dumps(event)}")
    for record in event["Records"]:
        LOGGER.debug(f"Processing record: {json.dumps(record)}")
        if "Sns" in record:
            message = json.loads(record["Sns"]["Message"])
            transition = message["LifecycleTransition"]

            if "NotificationMetadata" in message:
                notification_metadata = json.loads(message["NotificationMetadata"])
                hosted_zone_id = notification_metadata["HostedZoneId"]
                dns_name = notification_metadata["DnsName"]
                prefix = notification_metadata["Prefix"]
                autoscaling_group_name = notification_metadata["AsgName"]
            else:
                LOGGER.warn(
                    f"NotificationMetadata absent from message: {json.dumps(message)}"
                )

            if "EC2InstanceId" in message:
                instance_id = message["EC2InstanceId"]
                ip_address = _instance_ip_address(instance_id)
            else:
                LOGGER.warn(f"EC2InstanceId absent from message: {json.dumps(message)}")

            if transition == INSTANCE_LAUNCHING:
                try:
                    _insert_dns_ip(dns_name, ip_address, hosted_zone_id)
                finally:
                    _complete_lifecycle_action(
                        lifecycle_hook_name=f"{prefix}-SwarmLaunchHook",
                        autoscaling_group_name=autoscaling_group_name,
                        instance_id=instance_id,
                    )
            elif transition == INSTANCE_TERMINATING:
                try:
                    _remove_dns_ip(dns_name, ip_address, hosted_zone_id)
                finally:
                    _complete_lifecycle_action(
                        lifecycle_hook_name=f"{prefix}-SwarmTerminateHook",
                        autoscaling_group_name=autoscaling_group_name,
                        instance_id=instance_id,
                    )
            else:
                LOGGER.warn(
                    f'Encountered unknown lifecycle transition "{transition}" in message: {json.dumps(message)}'
                )
        else:
            LOGGER.warn(f"Encountered unknown record: {json.dumps(record)}")
