from infra import dynamodb
from infra.cache import Cache
from infra.config import configurable_envvars, repository_path
from infra.dynamodb import DynamoDB
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
from infra.metric_forwarder import MetricForwarder
from infra.network import Network


class NodeIdentifier(FargateService):
    def __init__(
        self,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        db: DynamoDB,
        network: Network,
        cache: Cache,
        forwarder: MetricForwarder,
    ) -> None:

        super().__init__(
            "node-identifier",
            image=GraplDockerBuild(
                dockerfile=str(repository_path("src/rust/Dockerfile")),
                target="node-identifier-deploy",
                context=str(repository_path("src")),
            ),
            retry_image=GraplDockerBuild(
                dockerfile=str(repository_path("src/rust/Dockerfile")),
                target="node-identifier-retry-deploy",
                context=str(repository_path("src")),
            ),
            env={
                **configurable_envvars(
                    "node-identifier", ["RUST_LOG", "RUST_BACKTRACE"]
                ),
                "REDIS_ENDPOINT": cache.endpoint,
                # TODO: If the retry handler doesn't get permission to
                # interact with these tables, then it probably
                # shouldn't get these environment variables.
                "GRAPL_STATIC_MAPPING_TABLE": db.static_mapping_table.name,
                "GRAPL_DYNAMIC_SESSION_TABLE": db.dynamic_session_table.name,
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )

        # Also, these are the same tables that were passed to the
        # service via environment variables above.
        for svc in self.services:
            dynamodb.grant_read_write_on_tables(
                svc.task_role,
                [
                    db.static_mapping_table,
                    db.dynamic_session_table,
                    db.process_history_table,
                    db.file_history_table,
                    db.inbound_connection_history_table,
                    db.outbound_connection_history_table,
                    db.network_connection_history_table,
                    db.ip_connection_history_table,
                ],
            )

        self.allow_egress_to_cache(cache)
