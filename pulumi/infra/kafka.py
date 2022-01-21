from __future__ import annotations

import dataclasses
import os
from typing import Any, Iterable, Mapping, Optional, Sequence, Tuple, cast

from infra.config import LOCAL_GRAPL
from pulumi.stack_reference import StackReference

import pulumi

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
class Credential:
    service_account_id: str
    api_key: str
    api_secret: str

    @staticmethod
    def from_json(json_: Mapping[str, str]) -> Credential:
        return Credential(
            service_account_id=json_["service_account_id"],
            api_key=json_["api_key"],
            api_secret=json_["api_secret"],
        )


@dataclasses.dataclass
class TopicOutput:
    partitions: int
    config: Mapping[str, Any]

    @staticmethod
    def from_json(json_: Mapping[str, Any]) -> TopicOutput:
        return TopicOutput(
            partitions=json_["partitions"],
            config=json_["config"],
        )


@dataclasses.dataclass
class Service:
    ingress_topics: Sequence[str]
    egress_topics: Sequence[str]
    service_account: Credential

    @staticmethod
    def from_json(json_: Mapping[str, Any]) -> Service:
        return Service(
            ingress_topics=json_["ingress_topics"],
            egress_topics=json_["egress_topics"],
            service_account=Credential.from_json(json_["service_account"]),
        )


@dataclasses.dataclass
class EnvironmentOutput:
    environment_id: str
    bootstrap_servers: str
    environment_credentials: Credential
    services: Mapping[str, Service]
    topics: Mapping[str, TopicOutput]

    def get_service_credentials(self, service_name: str) -> Credential:
        if service_name in self.services:
            return self.services[service_name].service_account
        else:
            raise KeyError(f"{service_name} does not exist")

    @staticmethod
    def from_json(json_: Mapping[str, Any]) -> EnvironmentOutput:
        return EnvironmentOutput(
            environment_id=json_["environment_id"],
            bootstrap_servers=json_["bootstrap_servers"],
            environment_credentials=Credential.from_json(
                json_["environment_credentials"]
            ),
            services={
                k: Service.from_json(v) for k, v in json_["services"].items()
            },
            topics={k: TopicOutput.from_json(v) for k, v in json_["topics"].items()},
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
    def from_json(
        json_: pulumi.Output[Mapping[str, Any]]
    ) -> pulumi.Output[ConfluentOutput]:
        return json_.apply(
            lambda j: ConfluentOutput(
                environments={k: EnvironmentOutput.from_json(v) for k, v in j.items()}
            )
        )


class Kafka(pulumi.ComponentResource):
    confluent_environment: pulumi.Output[EnvironmentOutput]

    def __init__(
        self,
        name: str,
        confluent_environment_name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ):
        super().__init__("grapl:Kafka", name=name, props=None, opts=opts)

        confluent_stack_output = StackReference(
            "grapl/ccloud-bootstrap/ccloud-bootstrap"
        ).require_output("confluent")

        self.confluent_environment = ConfluentOutput.from_json(
            cast(pulumi.Output[Mapping[str, Any]], confluent_stack_output)
        ).apply(lambda o: o.get_environment(confluent_environment_name))
