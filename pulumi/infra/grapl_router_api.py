from infra.fargate_service import FargateService, GraplDockerBuild
from infra.emitter import EventEmitter
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.cache import Cache
from infra.config import configurable_envvars



class GraplRouterApi(FargateService):
    def __init__(
            self,
            input_emitter: EventEmitter,
            output_emitter: EventEmitter,
            network: Network,
            cache: Cache,
            forwarder: MetricForwarder,
    ) -> None:

        super().__init__(
            "graph-merger",
            image=GraplDockerBuild(
                dockerfile="../src/rust/Dockerfile",
                target="grapl-router-api-deploy",  # check this
                context="../src",
            ),
            retry_image=GraplDockerBuild(
                dockerfile="../src/rust/Dockerfile",
                target="grapl-router-api-retry-handler-deploy",
                context="../src",
            ),
            command="/grapl-router-api",
            env={
                **configurable_envvars("grapl_router_api", ["RUST_LOG", "RUST_BACKTRACE"]),
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )

