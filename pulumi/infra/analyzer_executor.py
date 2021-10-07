from infra.bucket import Bucket
from infra.cache import Cache
from infra.config import configurable_envvars, repository_path
from infra.dgraph_cluster import DgraphCluster
from infra.emitter import EventEmitter
from infra.fargate_service import FargateService, GraplDockerBuild
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
    ) -> None:

        super().__init__(
            "analyzer-executor",
            image=GraplDockerBuild(
                # Should match entry in `docker-compose.build.yml`
                dockerfile=str(repository_path("src/python/Dockerfile")),
                target="analyzer-executor-deploy",
                context=str(repository_path(".")),
            ),
            env={
                **configurable_envvars("analyzer-executor", ["GRAPL_LOG_LEVEL"]),
                "GRAPL_ANALYZERS_BUCKET": analyzers_bucket.bucket,
                "GRAPL_MODEL_PLUGINS_BUCKET": model_plugins_bucket.bucket,
                "GRAPL_ANALYZER_MATCHED_SUBGRAPHS_BUCKET": output_emitter.bucket.bucket,
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
            network=network,
        )

        for svc in self.services:
            dgraph_cluster.allow_connections_from(svc.security_group)
            analyzers_bucket.grant_get_and_list_to(svc.task_role)
            model_plugins_bucket.grant_get_and_list_to(svc.task_role)
            output_emitter.grant_read_to(svc.task_role)

        self.allow_egress_to_cache(cache)
