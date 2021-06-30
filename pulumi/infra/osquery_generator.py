from infra.cache import Cache
from infra.config import configurable_envvars, repository_path
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
from infra.metric_forwarder import MetricForwarder
from infra.network import Network


class OSQueryGenerator(FargateService):
    def __init__(
        self,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        network: Network,
        cache: Cache,
        forwarder: MetricForwarder,
    ) -> None:
        super().__init__(
            "osquery-generator",
            image=GraplDockerBuild(
                dockerfile=str(repository_path("src/rust/Dockerfile")),
                target="osquery-subgraph-generator-deploy",
                context=str(repository_path("src")),
            ),
            command="/osquery-subgraph-generator",
            env={
                **configurable_envvars(
                    "osquery-generator", ["RUST_LOG", "RUST_BACKTRACE"]
                ),
                "REDIS_ENDPOINT": cache.endpoint,
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )

        self.allow_egress_to_cache(cache)
