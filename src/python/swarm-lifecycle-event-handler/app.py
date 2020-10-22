import json
import logger
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


def _remove_dns_ip(dns_name: str, ip_address: str, hosted_zone_id: str) -> str:
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


def _insert_dns_ip(dns_name: str, ip_address: str, hosted_zone_id: str) -> str:
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
    event_json = json.loads(event)
    transition = event_json["LifecycleTransition"]
    notification_metadata = json.loads(event_json["NotificationMetadata"])
    hosted_zone_id = notification_metadata["HostedZoneId"]
    dns_name = notification_metadata["DnsName"]
    instance_id = event_json["EC2InstanceId"]
    ip_address = _instance_ip_address(instance_id)
    if transition == INSTANCE_LAUNCHING:
        _insert_dns_ip(dns_name, ip_address, hosed_zone_id)
    elif transition == INSTANCE_TERMINATING:
        _remove_dns_ip(dns_name, ip_address, hosted_zone_id)
    else:
        LOGGER.warn(
            f"Encountered unknown lifecycle transition "{transition}" in event: {event_json}"
        )
