from typing import Mapping, Optional, Union
from pulumi.resource import ResourceOptions

import pulumi_kafka as kafka

import pulumi

from infra.config import LOCAL_GRAPL


class Kafka(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        confluent: Mapping[
            str, Union[pulumi.Output[str], Mapping[str, pulumi.Output[str]]]
        ],
        opts: Optional[pulumi.ResourceOptions] = None,
    ):
        super().__init__("grapl:Kafka", name=name, props=None, opts=opts)

        if LOCAL_GRAPL:
            provider = kafka.Provider(
                "kafka-provider",
                bootstrap_servers=[confluent["bootstrap_servers"]],
                opts=opts,
                tls_enabled=False,
            )
        else:
            provider = kafka.Provider(
                "kafka-provider",
                bootstrap_servers=[confluent["bootstrap_servers"]],
                opts=opts,
                sasl_mechanism="plain",
                sasl_password=confluent["grapl_pulumi"]["api_key"],
                sasl_username=confluent["grapl_pulumi"]["api_secret"],
                tls_enabled=True,
            )

        #
        # metrics topic
        #

        kafka.Topic(
            "metrics-topic",
            opts=pulumi.ResourceOptions(provider=provider),
            name="metrics",
            replication_factor=1,
            partitions=1,
        )

        if not LOCAL_GRAPL:
            services = set(confluent.keys()).difference(
                {"bootstrap_servers", "grapl_pulumi"}
            )
            for service in services:
                # give every service write access to the metrics topic
                kafka.Acl(
                    f"{service}-metrics-topic-acl",
                    opts=pulumi.ResourceOptions(provider=provider),
                    acl_resource_name="metrics",
                    acl_resource_type="Topic",
                    acl_principal=f"User:{confluent[service]['service_account_id']}",
                    acl_host="*",
                    acl_operation="Write",
                    acl_permission_type="Allow",
                )
