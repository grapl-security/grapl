from infra.config import configurable_envvars, repository_path
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
from infra.metric_forwarder import MetricForwarder
from infra.network import Network


class WebUI(FargateService):
    def __init__(
        self,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        network: Network,
        forwarder: MetricForwarder,
    ) -> None:
        super().__init__(
            "web-ui",
            image=GraplDockerBuild(
                dockerfile=str(repository_path("src/rust/Dockerfile")),
                target="grapl-web-ui",
                context=str(repository_path("src")),
            ),
            env={
                **configurable_envvars("web-ui", ["RUST_LOG", "RUST_BACKTRACE"]),
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )
