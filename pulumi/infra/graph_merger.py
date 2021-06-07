from infra import dynamodb
from infra.cache import Cache
from infra.config import configurable_envvars
from infra.dgraph_cluster import DgraphCluster
from infra.dynamodb import DynamoDB
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
from infra.graph_mutation_cluster import GraphMutationCluster
from infra.metric_forwarder import MetricForwarder
from infra.network import Network


class GraphMerger(FargateService):
    def __init__(
        self,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        dgraph_cluster: DgraphCluster,
        graph_mutation_service_cluster: GraphMutationCluster,
        network: Network,
        cache: Cache,
        forwarder: MetricForwarder,
    ) -> None:

        super().__init__(
            "graph-merger",
            image=GraplDockerBuild(
                dockerfile="../src/rust/Dockerfile",
                target="graph-merger-deploy",
                context="../src",
            ),
            command="/graph-merger",
            env={
                **configurable_envvars("graph-merger", ["RUST_LOG", "RUST_BACKTRACE"]),
                "REDIS_ENDPOINT": cache.endpoint,
                "MG_ALPHAS": dgraph_cluster.alpha_host_port,
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )

        dgraph_cluster.allow_connections_from(self.default_service.security_group)
        dgraph_cluster.allow_connections_from(self.retry_service.security_group)
        graph_mutation_service_cluster.allow_connections_from(
            self.default_service.security_group
        )
        graph_mutation_service_cluster.allow_connections_from(
            self.retry_service.security_group
        )

        # TODO: Interestingly, the CDK code doesn't have this, even though
        # the other services do.
        # self.allow_egress_to_cache(cache)
