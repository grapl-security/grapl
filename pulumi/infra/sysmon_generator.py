import pulumi_aws as aws
import pulumi_docker as docker
from infra.cache import Cache
from infra.config import configurable_envvars
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService
from infra.metric_forwarder import MetricForwarder
from infra.network import Network


class SysmonGenerator(FargateService):
    def __init__(
        self,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        network: Network,
        cache: Cache,
        forwarder: MetricForwarder,
    ) -> None:

        super().__init__(
            "sysmon-generator",
            image=docker.DockerBuild(
                dockerfile="../src/rust/Dockerfile",
                target="sysmon-subgraph-generator-deploy",
                context="../src",
                args={"RUST_BUILD": "debug"},
                env={"DOCKER_BUILDKIT": "1"},
            ),
            command="/sysmon-subgraph-generator",
            env={
                **configurable_envvars(
                    "sysmon-generator", ["RUST_LOG", "RUST_BACKTRACE"]
                ),
                "AWS_REGION": aws.get_region().name,
                "REDIS_ENDPOINT": cache.endpoint,
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )

        self.allow_egress_to_cache(cache)
