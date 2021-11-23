from __future__ import annotations

import dataclasses
import os
from typing import Any, Iterable, Mapping, Optional, Tuple

import pulumi_kafka as kafka
from infra.config import LOCAL_GRAPL

import pulumi


@dataclasses.dataclass
class CredentialOutput:
    service_account_id: str
    api_key: str
    api_secret: str

    @staticmethod
    async def from_json(json_: Mapping[str, str]) -> CredentialOutput:
        return CredentialOutput(
            service_account_id=json_["service_account_id"],
            api_key=json_["api_key"],
            api_secret=pulumi.Output.unsecret(json_["api_secret"]),
        )


@dataclasses.dataclass
class EnvironmentOutput:
    environment_id: str
    bootstrap_servers: str
    environment_credentials: CredentialOutput
    service_credentials: Mapping[str, CredentialOutput]

    @staticmethod
    def from_json(json_: pulumi.Output[Mapping[str, Any]]) -> EnvironmentOutput:
        return EnvironmentOutput(
            environment_id=json_["environment_id"],
            bootstrap_servers=json_["bootstrap_servers"],
            environment_credentials=CredentialOutput.from_json(
                json_["environment_credentials"]
            ),
            service_credentials={
                k: CredentialOutput.from_json(v)
                for k, v in json_["service_credentials"].items()
            },
        )


@dataclasses.dataclass
class ConfluentOutput:
    environments: Mapping[str, EnvironmentOutput]

    def get_environment(self, environment_name: str) -> EnvironmentOutput:
        if environment_name in self.environments:
            return self.environments[environment_name]
        else:
            raise KeyError(f"{environment_name} does not exist")

    @staticmethod
    def from_json(json_: pulumi.Output[Mapping[str, Any]]) -> ConfluentOutput:
        return ConfluentOutput(
            environments={
                k: EnvironmentOutput.from_json(v) for k, v in json_["environemnts"]
            }
        )


class Kafka(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        confluent_environment_name: str,
        confluent: ConfluentOutput,
        opts: Optional[pulumi.ResourceOptions] = None,
    ):
        super().__init__("grapl:Kafka", name=name, props=None, opts=opts)

        confluent_environment = confluent.get_environment(confluent_environment_name)

        if LOCAL_GRAPL:
            provider = kafka.Provider(
                "kafka-provider",
                bootstrap_servers=[os.environ["KAFKA_ENDPOINT"]],
                opts=opts,
                tls_enabled=False,
            )
        else:
            provider = kafka.Provider(
                "kafka-provider",
                bootstrap_servers=[confluent_environment.bootstrap_servers],
                opts=opts,
                sasl_mechanism="plain",
                sasl_username=confluent_environment.environment_credentials.service_account_id,
                sasl_password=confluent_environment.environment_credentials.api_secret,
                tls_enabled=True,
                timeout=60,
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
            for (
                service,
                credentials,
            ) in confluent_environment.service_credentials.items():
                # give every service write access to the metrics topic
                kafka.Acl(
                    f"{service}-metrics-topic-acl",
                    opts=pulumi.ResourceOptions(provider=provider),
                    acl_resource_name="metrics",
                    acl_resource_type="Topic",
                    acl_principal=f"User:{credentials.service_account_id}",
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
            for (
                service,
                credentials,
            ) in confluent_environment.service_credentials.items():
                # give every service write access to the logs topic
                kafka.Acl(
                    f"{service}-logs-topic-acl",
                    opts=pulumi.ResourceOptions(provider=provider),
                    acl_resource_name="logs",
                    acl_resource_type="Topic",
                    acl_principal=f"User:{credentials.service_account_id}",
                    acl_host="*",
                    acl_operation="Write",
                    acl_permission_type="Allow",
                )

        #
        # pipeline service topics
        #

        # pipeline-ingress

        pipeline_ingress_topic = kafka.Topic(
            "pipeline-ingress-topic",
            opts=pulumi.ResourceOptions(provider=provider),
            name="pipeline-ingress",
            config={},
            replication_factor=1,
            partitions=1,
        )

        pipeline_ingress_credentials = confluent_environment.service_credentials[
            "pipeline-ingress"
        ]

        self._create_acls(
            service_account_id=pipeline_ingress_credentials.service_account_id,
            topics=(pipeline_ingress_topic,),
            write=True,
            provider=provider,
        )

        # graph-generator

        graph_generator_credentials = confluent_environment.service_credentials[
            "graph-generator"
        ]

        self._create_acls(
            service_account_id=graph_generator_credentials.service_account_id,
            topics=(pipeline_ingress_topic,),
            write=False,
            provider=provider,
        )

        (
            graph_generator_topic,
            graph_generator_retry_topic,
            graph_generator_failed_topic,
        ) = self._create_topics(
            provider=provider,
            service="graph-generator",
            config={},
            replication_factor=1,
            partitions=1,
            retry_config={},
            retry_replication_factor=1,
            retry_partitions=1,
            failed_config={},
            failed_replication_factor=1,
            failed_partitions=1,
        )

        self._create_acls(
            service_account_id=graph_generator_credentials.service_account_id,
            topics=(
                graph_generator_topic,
                graph_generator_retry_topic,
                graph_generator_failed_topic,
            ),
            write=True,
            provider=provider,
        )

        # node-identifier

        node_identifier_credentials = confluent_environment.service_credentials[
            "node-identifier"
        ]

        self._create_acls(
            service_account_id=node_identifier_credentials.service_account_id,
            topics=(graph_generator_topic,),
            write=False,
            provider=provider,
        )

        (
            node_identifier_topic,
            node_identifier_retry_topic,
            node_identifier_failed_topic,
        ) = self._create_topics(
            provider=provider,
            service="node-identifier",
            config={},
            replication_factor=1,
            partitions=1,
            retry_config={},
            retry_replication_factor=1,
            retry_partitions=1,
            failed_config={},
            failed_replication_factor=1,
            failed_partitions=1,
        )

        self._create_acls(
            service_account_id=node_identifier_credentials.service_account_id,
            topics=(
                node_identifier_topic,
                node_identifier_retry_topic,
                node_identifier_failed_topic,
            ),
            write=True,
            provider=provider,
        )

        # graph-merger

        graph_merger_credentials = confluent_environment.service_credentials[
            "graph-merger"
        ]

        self._create_acls(
            service_account_id=graph_merger_credentials.service_account_id,
            topics=(node_identifier_topic,),
            write=False,
            provider=provider,
        )

        (
            graph_merger_topic,
            graph_merger_retry_topic,
            graph_merger_failed_topic,
        ) = self._create_topics(
            provider=provider,
            service="graph-merger",
            config={},
            replication_factor=1,
            partitions=1,
            retry_config={},
            retry_replication_factor=1,
            retry_partitions=1,
            failed_config={},
            failed_replication_factor=1,
            failed_partitions=1,
        )

        self._create_acls(
            service_account_id=graph_merger_credentials.service_account_id,
            topics=(
                graph_merger_topic,
                graph_merger_retry_topic,
                graph_merger_failed_topic,
            ),
            write=True,
            provider=provider,
        )

        # analyzer-executor

        analyzer_executor_credentials = confluent_environment.service_credentials[
            "analyzer-executor"
        ]

        self._create_acls(
            service_account_id=analyzer_executor_credentials.service_account_id,
            topics=(node_identifier_topic,),
            write=False,
            provider=provider,
        )

        (
            analyzer_executor_topic,
            analyzer_executor_retry_topic,
            analyzer_executor_failed_topic,
        ) = self._create_topics(
            provider=provider,
            service="analyzer-executor",
            config={},
            replication_factor=1,
            partitions=1,
            retry_config={},
            retry_replication_factor=1,
            retry_partitions=1,
            failed_config={},
            failed_replication_factor=1,
            failed_partitions=1,
        )

        self._create_acls(
            service_account_id=analyzer_executor_credentials.service_account_id,
            topics=(
                analyzer_executor_topic,
                analyzer_executor_retry_topic,
                analyzer_executor_failed_topic,
            ),
            write=True,
            provider=provider,
        )

        # engagement-creator

        engagement_creator_credentials = confluent_environment.service_credentials[
            "engagement-creator"
        ]

        self._create_acls(
            service_account_id=engagement_creator_credentials.service_account_id,
            topics=(analyzer_executor_topic,),
            write=False,
            provider=provider,
        )

        (
            engagement_creator_topic,
            engagement_creator_retry_topic,
            engagement_creator_failed_topic,
        ) = self._create_topics(
            provider=provider,
            service="engagement_creator",
            config={},
            replication_factor=1,
            partitions=1,
            retry_config={},
            retry_replication_factor=1,
            retry_partitions=1,
            failed_config={},
            failed_replication_factor=1,
            failed_partitions=1,
        )

        self._create_acls(
            service_account_id=engagement_creator_credentials.service_account_id,
            topics=(
                engagement_creator_topic,
                engagement_creator_retry_topic,
                engagement_creator_failed_topic,
            ),
            write=True,
            provider=provider,
        )

    @staticmethod
    def _create_topics(
        provider: kafka.Provider,
        service: str,
        config: Mapping[str, Any],
        replication_factor: int,
        partitions: int,
        retry_config: Mapping[str, Any],
        retry_replication_factor: int,
        retry_partitions: int,
        failed_config: Mapping[str, Any],
        failed_replication_factor: int,
        failed_partitions: int,
    ) -> Tuple[kafka.Topic, kafka.Topic, kafka.Topic]:
        topic = kafka.Topic(
            f"{service}-topic",
            opts=pulumi.ResourceOptions(provider=provider),
            name=service,
            config=config,
            replication_factor=replication_factor,
            partitions=partitions,
        )
        retry_topic = kafka.Topic(
            f"{service}-retry-topic",
            opts=pulumi.ResourceOptions(provider=provider),
            name=f"{service}-retry",
            config=retry_config,
            replication_factor=retry_replication_factor,
            partitions=retry_partitions,
        )
        failed_topic = kafka.Topic(
            f"{service}-failed-topic",
            opts=pulumi.ResourceOptions(provider=provider),
            name=f"{service}-failed",
            config=failed_config,
            replication_factor=failed_replication_factor,
            partitions=failed_partitions,
        )
        return topic, retry_topic, failed_topic

    @staticmethod
    def _create_acls(
        provider: kafka.Provider,
        service_account_id: str,
        topics: Iterable[kafka.Topic],
        write: bool = False,
    ) -> None:
        for topic in topics:
            kafka.Acl(
                f"{topic.name}-{service_account_id}-{'write' if write else 'read'}-acl",
                opts=pulumi.ResourceOptions(provider=provider),
                acl_resource_name=topic.name,
                acl_resource_type="Topic",
                acl_principal=f"User:{service_account_id}",
                acl_host="*",  # FIXME: restrict this?
                acl_operation="Write" if write else "Read",
                acl_permission_type="Allow",
            )
