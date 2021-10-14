from typing import Any, Mapping, Optional

import pulumi_kafka as kafka
from infra.config import LOCAL_GRAPL

import pulumi


class Kafka(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        confluent: Mapping[str, Any],
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
                sasl_username=confluent["management_api_key"],
                sasl_password=confluent["management_api_secret"],
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
            for service in confluent["service_credentials"].keys():
                # give every service write access to the metrics topic
                kafka.Acl(
                    f"{service}-metrics-topic-acl",
                    opts=pulumi.ResourceOptions(provider=provider),
                    acl_resource_name="metrics",
                    acl_resource_type="Topic",
                    acl_principal=f"User:{confluent['service_credentials'][service]['service_account_id']}",
                    acl_host="*",
                    acl_operation="Write",
                    acl_permission_type="Allow",
                )

        #
        # logs topic
        #

        kafka.Topic(
            "logs-topic",
            opts=pulumi.ResourceOptions(provider=provider),
            name="logs",
            replication_factor=1,
            partitions=1,
        )

        if not LOCAL_GRAPL:
            for service in confluent["service_credentials"].keys():
                # give every service write access to the logs topic
                kafka.Acl(
                    f"{service}-logs-topic-acl",
                    opts=pulumi.ResourceOptions(provider=provider),
                    acl_resource_name="logs",
                    acl_resource_type="Topic",
                    acl_principal=f"User:{confluent['service_credentials'][service]['service_account_id']}",
                    acl_host="*",
                    acl_operation="Write",
                    acl_permission_type="Allow",
                )

        #
        # pipeline service topics
        #

        kafka.Topic(
            "pipeline-ingress-topic",
            opts=pulumi.ResourceOptions(provider=provider),
            name="pipeline-ingress",
            replication_factor=1,
            partitions=1,
        )

        kafka.Acl(
            f"pipeline-ingress-topic-acl",
            opts=pulumi.ResourceOptions(provider=provider),
            acl_resource_name="pipeline-ingress",
            acl_resource_type="Topic",
            acl_principal=f"User:{confluent['service_credentials']['pipeline-ingress']['service_account_id']}",
            acl_host="*",  # FIXME: restrict this
            acl_operation="Write",
            acl_permission_type="Allow",
        )

        kafka.Topic(
            "pipeline-ingress-retry-topic",
            opts=pulumi.ResourceOptions(provider=provider),
            name="pipeline-ingress-retry",
            replication_factor=1,
            partitions=1,
        )

        kafka.Acl(
            f"pipeline-ingress-retry-topic-acl",
            opts=pulumi.ResourceOptions(provider=provider),
            acl_resource_name="pipeline-ingress-retry",
            acl_resource_type="Topic",
            acl_principal=f"User:{confluent['service_credentials']['pipeline-ingress']['service_account_id']}",
            acl_host="*",  # FIXME: restrict this
            acl_operation="Write",
            acl_permission_type="Allow",
        )

        kafka.Topic(
            "pipeline-ingress-failed-topic",
            opts=pulumi.ResourceOptions(provider=provider),
            name="pipeline-ingress-failed",
            replication_factor=1,
            partitions=1,
        )

        kafka.Acl(
            f"pipeline-ingress-failed-topic-acl",
            opts=pulumi.ResourceOptions(provider=provider),
            acl_resource_name="pipeline-ingress-failed",
            acl_resource_type="Topic",
            acl_principal=f"User:{confluent['service_credentials']['pipeline-ingress']['service_account_id']}",
            acl_host="*",  # FIXME: restrict this
            acl_operation="Write",
            acl_permission_type="Allow",
        )

        # TODO: create pipeline topics as needed, with ACLs such that only the
        # services which need write access have write access and only the
        # services which need read access have read access.

    # TODO: write methods to provide per-service client credentials
