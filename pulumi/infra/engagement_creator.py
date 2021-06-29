from infra.config import configurable_envvars
from infra.dgraph_cluster import DgraphCluster
from infra.emitter import EventEmitter
from infra.lambda_ import code_path_for
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.service import Service


class EngagementCreator(Service):
    def __init__(
        self,
        input_emitter: EventEmitter,
        network: Network,
        forwarder: MetricForwarder,
        dgraph_cluster: DgraphCluster,
    ) -> None:

        name = "engagement-creator"
        super().__init__(
            name,
            forwarder=forwarder,
            lambda_handler_fn="lambdex_handler.handler",
            lambda_code_path=code_path_for("engagement-creator"),
            network=network,
            env={
                **configurable_envvars(name, ["GRAPL_LOG_LEVEL"]),
                "MG_ALPHAS": dgraph_cluster.alpha_host_port,
            },
        )

        self.queue.subscribe_to_emitter(input_emitter)
        input_emitter.grant_read_to(self.role)

        for handler in self.handlers:
            dgraph_cluster.allow_connections_from(handler.function.security_group)

        self.register_outputs({})
