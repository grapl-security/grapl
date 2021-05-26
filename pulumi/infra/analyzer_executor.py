import pulumi_docker as docker
from infra.bucket import Bucket
from infra.cache import Cache
from infra.config import configurable_envvars
from infra.dgraph_cluster import DgraphCluster
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
from infra.metric_forwarder import MetricForwarder
from infra.network import Network


class AnalyzerExecutor(FargateService):
    def __init__(
        self,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        dgraph_cluster: DgraphCluster,
        analyzers_bucket: Bucket,
        model_plugins_bucket: Bucket,
        network: Network,
        cache: Cache,
        forwarder: MetricForwarder,
    ) -> None:

        super().__init__(
            "analyzer-executor",
            image=GraplDockerBuild(
                dockerfile="../src/python/Dockerfile",
                target="analyzer-executor-deploy",
                context="../src",
            ),
            command="/analyzer-executor",
            env={
                **configurable_envvars("analyzer-executor", ["GRAPL_LOG_LEVEL"]),
                "ANALYZER_MATCH_BUCKET": analyzers_bucket.bucket,
                "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                # TODO: We should modify this to use REDIS_ENDPOINT,
                # like our other services.
                # TODO: We should consolidate these different "caches"
                # into one internally.
                "MESSAGECACHE_ADDR": cache.host,
                "MESSAGECACHE_PORT": cache.port.apply(str),
                "HITCACHE_ADDR": cache.host,
                "HITCACHE_PORT": cache.port.apply(str),
                "GRPC_ENABLE_FORK_SUPPORT": "1",
            },
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            forwarder=forwarder,
            network=network,
        )

        dgraph_cluster.allow_connections_from(self.default_service.security_group)
        dgraph_cluster.allow_connections_from(self.retry_service.security_group)

        # TODO: These permissions are not granted to the retry
        # service; that appears to be a bug
        analyzers_bucket.grant_get_and_list_to(self.default_service.task_role)
        model_plugins_bucket.grant_get_and_list_to(self.default_service.task_role)
        output_emitter.bucket.grant_get_to(self.default_service.task_role)

        # TODO: CDK doesn't actually have this for some reason
        self.allow_egress_to_cache(cache)
