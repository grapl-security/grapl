from infra import dynamodb
from infra.cache import Cache
from infra.config import configurable_envvars
from infra.dgraph_cluster import DgraphCluster
from infra.dynamodb import DynamoDB
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
from infra.metric_forwarder import MetricForwarder
from infra.network import Network


class GraphMerger(FargateService):
    def __init__(
        self,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        dgraph_cluster: DgraphCluster,
        db: DynamoDB,
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
                "GRAPL_SCHEMA_TABLE": db.schema_table.name,
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )

        dgraph_cluster.allow_connections_from(self.default_service.security_group)
        dgraph_cluster.allow_connections_from(self.retry_service.security_group)

        # TODO: both the default and retry services get READ
        # permissions on the schema and schema properties table, even
        # though only the schema table was passed into the
        # environment.
        #
        # Investigate this further: is the properties table needed?
        for role in [self.default_service.task_role, self.retry_service.task_role]:
            dynamodb.grant_read_on_tables(
                role, [db.schema_table, db.schema_properties_table]
            )

        # TODO: Interestingly, the CDK code doesn't have this, even though
        # the other services do.
        # self.allow_egress_to_cache(cache)
