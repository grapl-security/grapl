from infra.cache import Cache
from infra.config import configurable_envvars
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
from infra.metric_forwarder import MetricForwarder
from infra.network import Network


class RouterApi(FargateService):
    def __init__(
        self,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        network: Network,
        cache: Cache,
        forwarder: MetricForwarder,
    ) -> None:

        super().__init__(
            "router-api",
            image=GraplDockerBuild(
                dockerfile="../src/rust/Dockerfile",
                target="router-api-deploy",  # check this
                context="../src",
            ),
            retry_image=GraplDockerBuild(
                dockerfile="../src/rust/Dockerfile",
                target="router-api-retry-handler-deploy",
                context="../src",
            ),
            command="/router-api",
            env={
                **configurable_envvars(
                    "router_api", ["RUST_LOG", "RUST_BACKTRACE"]
                ),
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )
