from infra.bucket import Bucket
from infra.cache import Cache
from infra.config import configurable_envvars, repository_path
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
from infra.metric_forwarder import MetricForwarder
from infra.network import Network


class AnalyzerDispatcher(FargateService):
    def __init__(
        self,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        analyzers_bucket: Bucket,
        network: Network,
        cache: Cache,
        forwarder: MetricForwarder,
    ) -> None:

        super().__init__(
            "analyzer-dispatcher",
            image=GraplDockerBuild(
                dockerfile=str(repository_path("src/rust/Dockerfile")),
                target="analyzer-dispatcher-deploy",
                context=str(repository_path("src")),
            ),
            env={
                **configurable_envvars(
                    "analyzer-dispatcher", ["RUST_LOG", "RUST_BACKTRACE"]
                ),
                "REDIS_ENDPOINT": cache.endpoint,
                "GRAPL_ANALYZERS_BUCKET": analyzers_bucket.bucket,
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )

        for svc in self.services:
            analyzers_bucket.grant_get_and_list_to(svc.task_role)

        self.allow_egress_to_cache(cache)
