from __future__ import annotations

import dataclasses
from typing import Any, Mapping, Optional, Sequence, cast

from pulumi.stack_reference import StackReference

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

    @staticmethod
    def from_json(data: Mapping[str, Any]) -> Service:
        return Service(
            ingress_topics=data["ingress_topics"],
            egress_topics=data["egress_topics"],
            service_account=Credential.from_json(data["service_account"]),
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
    ):
        super().__init__("grapl:Kafka", name=name, props=None, opts=opts)

        if confluent_environment_name == "local-grapl":
            self.confluent_environment = None
        else:
            confluent_stack_output = StackReference(
                "grapl/ccloud-bootstrap/ccloud-bootstrap"
            ).require_output("confluent")
            self.confluent_environment = Confluent.from_json(
                cast(pulumi.Output[Mapping[str, Any]], confluent_stack_output)
            ).apply(lambda o: o.get_environment(confluent_environment_name))

    def bootstrap_servers(self) -> pulumi.Output[str]:
        if self.confluent_environment is None: # local-grapl
            return pulumi.Output.from_input("LOCAL_GRAPL_REPLACE_IP:19092")
        else:
            return self.confluent_environment.apply(lambda e: e.bootstrap_servers)
