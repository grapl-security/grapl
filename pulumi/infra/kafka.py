from __future__ import annotations

import dataclasses
import os
from typing import Any, Iterable, Mapping, Optional, Tuple, cast

import pulumi_kafka as kafka
from infra.config import LOCAL_GRAPL

import pulumi
from pulumi.stack_reference import StackReference

# this list of service names must match those in the
# confluent-cloud-infrastructure project:
# TODO: link
KAFKA_SERVICES = [
    "pipeline-ingress",
    "graph-generator",
    "node-identifier",
    "graph-merger",
    "analyzer-executor",
    "engagement-creator",
]


@dataclasses.dataclass
class CredentialOutput:
    service_account_id: str
    api_key: str
    api_secret: str

    @staticmethod
    def from_json(json_: Mapping[str, str]) -> CredentialOutput:
        return CredentialOutput(
            service_account_id=json_["service_account_id"],
            api_key=json_["api_key"],
            api_secret=json_["api_secret"],
        )


@dataclasses.dataclass
class EnvironmentOutput:
    environment_id: str
    bootstrap_servers: str
    environment_credentials: CredentialOutput
    service_credentials: Mapping[str, CredentialOutput]

    def get_service_credentials(self, service_name: str) -> CredentialOutput:
        if service_name in self.service_credentials:
            return self.service_credentials[service_name]
        else:
            raise KeyError(f"{service_name} does not exist")

    @staticmethod
    def from_json(json_: Mapping[str, Any]) -> EnvironmentOutput:
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
    def from_json(json_: pulumi.Output[Mapping[str, Any]]) -> pulumi.Output[ConfluentOutput]:
        return json_.apply(
            lambda j: ConfluentOutput(
                environments={
                    k: EnvironmentOutput.from_json(v) for k, v in j.items()
                }
            )
        )


class Kafka(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        confluent_environment_name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ):
        super().__init__("grapl:Kafka", name=name, props=None, opts=opts)

        confluent_stack_output = StackReference(
            "grapl/ccloud-bootstrap/ccloud-bootstrap"
        ).get_output("confluent")

        assert confluent_stack_output is not None

        confluent_environment = ConfluentOutput.from_json(
            cast(pulumi.Output[Mapping[str, Any]], confluent_stack_output)
        ).apply(lambda o: o.get_environment(confluent_environment_name))

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
                bootstrap_servers=confluent_environment.apply(
                    lambda e: [e.bootstrap_servers]
                ),
                opts=opts,
                sasl_mechanism="plain",
                sasl_username=confluent_environment.apply(
                    lambda e: e.environment_credentials.service_account_id
                ),
                sasl_password=confluent_environment.apply(
                    lambda e: e.environment_credentials.api_secret
                ),
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
            for service_name in KAFKA_SERVICES:
                service_credentials = confluent_environment.apply(
                    lambda e: e.get_service_credentials(service_name)
                )
                # give every service write access to the metrics topic
                kafka.Acl(
                    f"{service_name}-metrics-topic-acl",
                    opts=pulumi.ResourceOptions(provider=provider),
                    acl_resource_name="metrics",
                    acl_resource_type="Topic",
                    acl_principal=service_credentials.apply(
                        lambda c: f"User:{c.service_account_id}"
                    ),
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
            for service_name in KAFKA_SERVICES:
                service_credentials = confluent_environment.apply(
                    lambda e: e.get_service_credentials(service_name)
                )
                # give every service write access to the logs topic
                kafka.Acl(
                    f"{service_name}-logs-topic-acl",
                    opts=pulumi.ResourceOptions(provider=provider),
                    acl_resource_name="logs",
                    acl_resource_type="Topic",
                    acl_principal=service_credentials.apply(
                        lambda c: f"User:{c.service_account_id}"
                    ),
                    acl_host="*",
                    acl_operation="Write",
                    acl_permission_type="Allow",
                )

        #
        # pipeline service topics
        #

        # pipeline-ingress

        assert "pipeline-ingress" in KAFKA_SERVICES

        pipeline_ingress_topic = kafka.Topic(
            "pipeline-ingress-topic",
            opts=pulumi.ResourceOptions(provider=provider),
            name="pipeline-ingress",
            config={},
            replication_factor=1,
            partitions=1,
        )

        pipeline_ingress_credentials = confluent_environment.apply(
            lambda e: e.get_service_credentials("pipeline-ingress")
        )

        self._create_acls(
            service_name="pipeline-ingress",
            service_credentials=pipeline_ingress_credentials,
            topics=(pipeline_ingress_topic,),
            write=True,
            provider=provider,
        )

        # graph-generator

        assert "graph-generator" in KAFKA_SERVICES

        graph_generator_credentials = confluent_environment.apply(
            lambda e: e.get_service_credentials("graph-generator")
        )

        self._create_acls(
            service_name="graph-generator",
            service_credentials=graph_generator_credentials,
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
            service_name="graph-generator",
            service_credentials=graph_generator_credentials,
            topics=(
                graph_generator_topic,
                graph_generator_retry_topic,
                graph_generator_failed_topic,
            ),
            write=True,
            provider=provider,
        )

        # node-identifier

        assert "node-identifier" in KAFKA_SERVICES

        node_identifier_credentials = confluent_environment.apply(
            lambda e: e.get_service_credentials("node-identifier")
        )

        self._create_acls(
            service_name="node-identifier",
            service_credentials=node_identifier_credentials,
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
            service_name="node-identifier",
            service_credentials=node_identifier_credentials,
            topics=(
                node_identifier_topic,
                node_identifier_retry_topic,
                node_identifier_failed_topic,
            ),
            write=True,
            provider=provider,
        )

        # graph-merger

        assert "graph-merger" in KAFKA_SERVICES

        graph_merger_credentials = confluent_environment.apply(
            lambda e: e.get_service_credentials("graph-merger")
        )

        self._create_acls(
            service_name="graph-merger",
            service_credentials=graph_merger_credentials,
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
            service_name="graph-merger",
            service_credentials=graph_merger_credentials,
            topics=(
                graph_merger_topic,
                graph_merger_retry_topic,
                graph_merger_failed_topic,
            ),
            write=True,
            provider=provider,
        )

        # analyzer-executor

        assert "analyzer-executor" in KAFKA_SERVICES

        analyzer_executor_credentials = confluent_environment.apply(
            lambda e: e.get_service_credentials("analyzer-executor")
        )

        self._create_acls(
            service_name="analyzer-executor",
            service_credentials=analyzer_executor_credentials,
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
            service_name="analyzer-executor",
            service_credentials=analyzer_executor_credentials,
            topics=(
                analyzer_executor_topic,
                analyzer_executor_retry_topic,
                analyzer_executor_failed_topic,
            ),
            write=True,
            provider=provider,
        )

        # engagement-creator

        assert "engagement-creator" in KAFKA_SERVICES

        engagement_creator_credentials = confluent_environment.apply(
            lambda e: e.get_service_credentials("engagement-creator")
        )

        self._create_acls(
            service_name="engagement-creator",
            service_credentials=engagement_creator_credentials,
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
            service_name="engagement-creator",
            service_credentials=engagement_creator_credentials,
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
        service_name: str,
        service_credentials: pulumi.Output[CredentialOutput],
        topics: Iterable[kafka.Topic],
        write: bool = False,
    ) -> None:
        for topic in topics:
            kafka.Acl(
                f"{topic.name}-{service_name}-{'write' if write else 'read'}-acl",
                opts=pulumi.ResourceOptions(provider=provider),
                acl_resource_name=topic.name,
                acl_resource_type="Topic",
                acl_principal=service_credentials.apply(
                    lambda c: f"User:{c.service_account_id}"
                ),
                acl_host="*",  # FIXME: restrict this?
                acl_operation="Write" if write else "Read",
                acl_permission_type="Allow",
            )
