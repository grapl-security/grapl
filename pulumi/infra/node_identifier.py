import pulumi_docker as docker
from infra.cache import Cache
from infra.config import configurable_envvars
from infra.dynamodb import DynamoDB
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService
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
            image=docker.DockerBuild(
                dockerfile="../src/rust/Dockerfile",
                target="node-identifier-deploy",
                context="../src",
                args={"RUST_BUILD": "debug"},
                env={"DOCKER_BUILDKIT": "1"},
            ),
            retry_image=docker.DockerBuild(
                dockerfile="../src/rust/Dockerfile",
                target="node-identifier-retry-handler-deploy",
                context="../src",
                args={"RUST_BUILD": "debug"},
                env={"DOCKER_BUILDKIT": "1"},
            ),
            command="/node-identifier",
            retry_command="/node-identifier-retry-handler",
            env={
                **configurable_envvars(
                    "node-identifier", ["RUST_LOG", "RUST_BACKTRACE"]
                ),
                "REDIS_ENDPOINT": cache.endpoint,
                # TODO: If the retry handler doesn't get permission to
                # interact with these tables, then it probably
                # shouldn't get these environment variables.
                "STATIC_MAPPING_TABLE": db.static_mapping_table.name,
                "DYNAMIC_SESSION_TABLE": db.dynamic_session_table.name,
                "PROCESS_HISTORY_TABLE": db.process_history_table.name,
                "FILE_HISTORY_TABLE": db.file_history_table.name,
                "INBOUND_CONNECTION_HISTORY_TABLE": db.inbound_connection_history_table.name,
                "OUTBOUND_CONNECTION_HISTORY_TABLE": db.outbound_connection_history_table.name,
                "NETWORK_CONNECTION_HISTORY_TABLE": db.network_connection_history_table.name,
                "IP_CONNECTION_HISTORY_TABLE": db.ip_connection_history_table.name,
                "ASSET_ID_MAPPINGS": db.asset_id_mappings.name,
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )

        for table in (
            db.static_mapping_table,
            db.dynamic_session_table,
            db.process_history_table,
            db.file_history_table,
            db.inbound_connection_history_table,
            db.outbound_connection_history_table,
            db.network_connection_history_table,
            db.ip_connection_history_table,
            db.asset_id_mappings,
        ):
            # Note that these permissions are only granted to the
            # default service's task role, *not* the retry service.
            #
            # Also, these are the same tables that were passed to the
            # service via environment variables above.
            table.grant_read_write_permissions_to(self.default_service.task_role)

        self.allow_egress_to_cache(cache)
