from __future__ import annotations

import dataclasses
from typing import Any, Mapping, Optional, Sequence, TypeVar, cast

from infra import config
from pulumi.stack_reference import StackReference
from pulumi_kafka import Provider
from pulumi_kafka import Topic as KafkaTopic

import pulumi


@dataclasses.dataclass
class Credential:
    service_account_id: str
    api_key: str
    api_secret: str

    @staticmethod
    def from_json(data: Mapping[str, str]) -> Credential:
        return Credential(
            service_account_id=data["service_account_id"],
            api_key=data["api_key"],
            api_secret=data["api_secret"],
        )


@dataclasses.dataclass
class Topic:
    partitions: int
    config: Mapping[str, Any]

    @staticmethod
    def from_json(data: Mapping[str, Any]) -> Topic:
        return Topic(
            partitions=data["partitions"],
            config=data["config"],
        )


@dataclasses.dataclass
class Service:
    ingress_topics: Sequence[str]
    egress_topics: Sequence[str]
    service_account: Credential
    consumer_group_name: Optional[str] = None

    @staticmethod
    def from_json(data: Mapping[str, Any]) -> Service:
        return Service(
            ingress_topics=data["ingress_topics"],
            egress_topics=data["egress_topics"],
            service_account=Credential.from_json(data["service_account"]),
            consumer_group_name=data.get("consumer_group_name"),
        )


@dataclasses.dataclass
class Environment:
    environment_id: str
    bootstrap_servers: str
    environment_credentials: Credential
    services: Mapping[str, Service]
    topics: Mapping[str, Topic]

    def get_service_credentials(self, service_name: str) -> Credential:
        if service_name in self.services:
            return self.services[service_name].service_account
        else:
            raise KeyError(f"{service_name} does not exist")

    @staticmethod
    def from_json(data: Mapping[str, Any]) -> Environment:
        return Environment(
            environment_id=data["environment_id"],
            bootstrap_servers=data["bootstrap_servers"],
            environment_credentials=Credential.from_json(
                data["environment_credentials"]
            ),
            services={k: Service.from_json(v) for k, v in data["services"].items()},
            topics={k: Topic.from_json(v) for k, v in data["topics"].items()},
        )


@dataclasses.dataclass
class Confluent:
    environments: Mapping[str, Environment]

    def get_environment(self, environment_name: str) -> Environment:
        if environment_name in self.environments:
            return self.environments[environment_name]
        else:
            raise KeyError(f"{environment_name} does not exist")

    @staticmethod
    def from_json(data: pulumi.Output[Mapping[str, Any]]) -> pulumi.Output[Confluent]:
        # Note this method takes a pulumi.Output[Mapping[str, Any]] whereas the
        # other from_json implementations in this file take just a bare
        # Mapping[str, Any]. The reason for this is that this class represents
        # the outermost layer of a nested object which is the stack output of
        # the "ccloud-bootstrap" stack. All the other from_json methods are
        # called recursively in the apply(..) call below.
        return data.apply(
            lambda j: Confluent(
                environments={k: Environment.from_json(v) for k, v in j.items()}
            )
        )


class Kafka(pulumi.ComponentResource):
    confluent_environment: Optional[pulumi.Output[Environment]]

    def __init__(
        self,
        name: str,
        confluent_environment_name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
        create_local_topics: bool = True,
    ):
        super().__init__("grapl:Kafka", name=name, props=None, opts=opts)

        if confluent_environment_name == "local-grapl":
            self.confluent_environment = None
            # This list must match the set of all topics specified in
            # confluent-cloud-infrastructure/pulumi/ccloud_bootstrap/index.ts
            topics = [
                "logs",
                "metrics",
                "raw-logs",
                "generated-graphs",
                "generated-graphs-retry",
                "generated-graphs-failed",
                "identified-graphs",
                "identified-graphs-retry",
                "identified-graphs-failed",
                "merged-graphs",
                "merged-graphs-retry",
                "merged-graphs-failed",
                "analyzer-executions",
                "analyzer-executions-retry",
                "analyzer-executions-failed",
                "engagements",
                "engagements-retry",
                "engagements-failed",
            ]
            provider = Provider(
                "grapl:kafka:Provider",
                bootstrap_servers=["host.docker.internal:29092"],
                tls_enabled=False,
            )
            if create_local_topics:
                for topic in topics:
                    KafkaTopic(
                        f"grapl:kafka:Topic:{topic}",
                        name=topic,
                        partitions=1,
                        replication_factor=1,
                        config={"compression.type": "zstd"},
                        opts=pulumi.ResourceOptions(provider=provider),
                    )
        else:
            confluent_stack_output = StackReference(
                "grapl/ccloud-bootstrap/ccloud-bootstrap"
            ).require_output("confluent")
            self.confluent_environment = Confluent.from_json(
                cast(pulumi.Output[Mapping[str, Any]], confluent_stack_output)
            ).apply(lambda o: o.get_environment(confluent_environment_name))

    def bootstrap_servers(self) -> pulumi.Output[str]:
        if self.confluent_environment is None:  # local-grapl
            return pulumi.Output.from_input(f"{config.HOST_IP_IN_NOMAD}:19092")
        else:
            return self.confluent_environment.apply(lambda e: e.bootstrap_servers)

    def service_credentials(self, service_name: str) -> pulumi.Output[Credential]:
        if self.confluent_environment is None:
            return pulumi.Output.from_input(
                Credential(
                    service_account_id="dummy_confluent_service_account_id",
                    api_key="dummy_confluent_api_key",
                    api_secret="dummy_confluent_api_secret",
                )
            )
        else:
            return self.confluent_environment.apply(
                lambda e: e.services[service_name].service_account
            )

    def consumer_group(self, service_name: str) -> pulumi.Output[str]:
        if self.confluent_environment is None:
            return pulumi.Output.from_input(service_name)
        else:
            return self.confluent_environment.apply(
                lambda e: _expect(e.services[service_name].consumer_group_name)
            )


T = TypeVar("T")


def _expect(val: Optional[T]) -> T:
    if val is None:
        raise Exception("expected value to be present")
    else:
        return val
