from infra.bucket import Bucket
from infra.cache import Cache
from infra.config import configurable_envvars
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
                dockerfile="../src/rust/Dockerfile",
                target="analyzer-dispatcher-deploy",
                context="../src",
            ),
            command="/analyzer-dispatcher",
            env={
                **configurable_envvars(
                    "analyzer-dispatcher", ["RUST_LOG", "RUST_BACKTRACE"]
                ),
                "REDIS_ENDPOINT": cache.endpoint,
                "ANALYZER_BUCKET": analyzers_bucket.bucket,
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )

        analyzers_bucket.grant_read_permissions_to(self.default_service.task_role)
        analyzers_bucket.grant_read_permissions_to(self.retry_service.task_role)

        self.allow_egress_to_cache(cache)
