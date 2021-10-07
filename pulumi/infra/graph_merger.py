from infra import dynamodb
from infra.cache import Cache
from infra.config import configurable_envvars, repository_path
from infra.dgraph_cluster import DgraphCluster
from infra.dynamodb import DynamoDB
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
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
    ) -> None:

        super().__init__(
            "graph-merger",
            image=GraplDockerBuild(
                dockerfile=str(repository_path("src/rust/Dockerfile")),
                target="graph-merger-deploy",
                context=str(repository_path("src")),
            ),
            env={
                **configurable_envvars("graph-merger", ["RUST_LOG", "RUST_BACKTRACE"]),
                "REDIS_ENDPOINT": cache.endpoint,
                "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                "GRAPL_SCHEMA_TABLE": db.schema_table.name,
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            network=network,
        )

        for service in self.services:
            dgraph_cluster.allow_connections_from(service.security_group)

            # TODO: both the default and retry services get READ
            # permissions on the schema and schema properties table, even
            # though only the schema table was passed into the
            # environment.
            #
            # Update (wimax Jun 2021): The properties table *is* needed, it was introduced
            # as part of the "python schema ---> json dynamodb ---> graphql schema"
            # pipeline.
            # It doesn't use the passed-in-via-environment mechanism because
            # of the `get_schema_properties_table` in common.
            # TODO: That function should be changed to instead depend on environment variables.
            dynamodb.grant_read_on_tables(
                service.task_role, [db.schema_table, db.schema_properties_table]
            )

        self.allow_egress_to_cache(cache)
