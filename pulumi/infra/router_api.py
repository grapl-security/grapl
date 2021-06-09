from infra.cache import Cache
from infra.config import configurable_envvars
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
from infra.metric_forwarder import MetricForwarder
from infra.network import Network


class WebUi(FargateService):
    def __init__(
        self,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        network: Network,
        cache: Cache,
        forwarder: MetricForwarder,
    ) -> None:

        super().__init__(
            "web-ui",
            image=GraplDockerBuild(
                dockerfile="../src/rust/Dockerfile",
                target="web-ui-deploy",  # check this
                context="../src",
            ),
            retry_image=GraplDockerBuild(
                dockerfile="../src/rust/Dockerfile",
                target="web-ui-retry-handler-deploy",
                context="../src",
            ),
            command="/web-ui",
            env={
                **configurable_envvars("web_ui", ["RUST_LOG", "RUST_BACKTRACE"]),
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )
