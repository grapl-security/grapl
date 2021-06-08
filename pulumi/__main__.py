import os
from pathlib import Path

from infra import dynamodb, emitter
from infra.alarms import OpsAlarms
from infra.analyzer_dispatcher import AnalyzerDispatcher
from infra.analyzer_executor import AnalyzerExecutor
from infra.api import Api
from infra.autotag import register_auto_tags
from infra.bucket import Bucket
from infra.cache import Cache
from infra.config import DEPLOYMENT_NAME, LOCAL_GRAPL
from infra.dgraph_cluster import DgraphCluster, LocalStandInDgraphCluster
from infra.dgraph_ttl import DGraphTTL
from infra.engagement_creator import EngagementCreator
from infra.graph_merger import GraphMerger
from infra.router_api import RouterApi
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.node_identifier import NodeIdentifier
from infra.osquery_generator import OSQueryGenerator
from infra.provision_lambda import Provisioner
from infra.secret import JWTSecret
from infra.sysmon_generator import SysmonGenerator


def _create_dgraph_cluster(network: Network) -> DgraphCluster:
    if LOCAL_GRAPL:
        return LocalStandInDgraphCluster()
    else:
        return DgraphCluster(
            name=f"{DEPLOYMENT_NAME}-dgraph",
            vpc=network.vpc,
        )


def main() -> None:

    if not LOCAL_GRAPL:
        # Fargate services build their own images and need this
        # variable currently. We don't want this to be checked in
        # Local Grapl, though.
        if not os.getenv("DOCKER_BUILDKIT"):
            raise KeyError("Please re-run with 'DOCKER_BUILDKIT=1'")

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": DEPLOYMENT_NAME})

    network = Network("grapl-network")

    dgraph_cluster: DgraphCluster = _create_dgraph_cluster(network=network)

    DGraphTTL(network=network, dgraph_cluster=dgraph_cluster)

    secret = JWTSecret()

    dynamodb_tables = dynamodb.DynamoDB()

    forwarder = MetricForwarder(network=network)

    # TODO: Create these emitters inside the service abstraction if nothing
    # else uses them (or perhaps even if something else *does* use them)
    sysmon_log_emitter = emitter.EventEmitter("sysmon-log")
    osquery_log_emitter = emitter.EventEmitter("osquery-log")
    unid_subgraphs_generated_emitter = emitter.EventEmitter("unid-subgraphs-generated")
    subgraphs_generated_emitter = emitter.EventEmitter("subgraphs-generated")
    subgraphs_merged_emitter = emitter.EventEmitter("subgraphs-merged")
    dispatched_analyzer_emitter = emitter.EventEmitter("dispatched-analyzer")
    analyzer_matched_emitter = emitter.EventEmitter("analyzer-matched-subgraphs")
    router_api = emitter.EventEmitter("router-api")

    # TODO: No _infrastructure_ currently *writes* to this bucket
    analyzers_bucket = Bucket("analyzers-bucket", sse=True)
    model_plugins_bucket = Bucket("model-plugins-bucket", sse=False)

    if LOCAL_GRAPL:
        # We need to create these queues, and wire them up to their
        # respective emitters, in Local Grapl, because they are
        # otherwise created in the FargateService instances below; we
        # don't run Fargate services in Local Grapl.
        #
        # T_T
        from infra.service_queue import ServiceQueue

        sysmon_generator_queue = ServiceQueue("sysmon-generator")
        sysmon_generator_queue.subscribe_to_emitter(sysmon_log_emitter)

        osquery_generator_queue = ServiceQueue("osquery-generator")
        osquery_generator_queue.subscribe_to_emitter(osquery_log_emitter)

        node_identifier_queue = ServiceQueue("node-identifier")
        node_identifier_queue.subscribe_to_emitter(unid_subgraphs_generated_emitter)

        graph_merger_queue = ServiceQueue("graph-merger")
        graph_merger_queue.subscribe_to_emitter(subgraphs_generated_emitter)

        analyzer_dispatcher_queue = ServiceQueue("analyzer-dispatcher")
        analyzer_dispatcher_queue.subscribe_to_emitter(subgraphs_merged_emitter)

        analyzer_executor_queue = ServiceQueue("analyzer-executor")
        analyzer_executor_queue.subscribe_to_emitter(dispatched_analyzer_emitter)

    else:
        # No Fargate or Elasticache in Local Grapl
        cache = Cache("main-cache", network=network)

        SysmonGenerator(
            input_emitter=sysmon_log_emitter,
            output_emitter=unid_subgraphs_generated_emitter,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        OSQueryGenerator(
            input_emitter=osquery_log_emitter,
            output_emitter=unid_subgraphs_generated_emitter,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        NodeIdentifier(
            input_emitter=unid_subgraphs_generated_emitter,
            output_emitter=subgraphs_generated_emitter,
            db=dynamodb_tables,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        GraplRouterApi(
            input_emitter=unid_subgraphs_generated_emitter,
            output_emitter=subgraphs_generated_emitter,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        GraphMerger(
            input_emitter=subgraphs_generated_emitter,
            output_emitter=subgraphs_merged_emitter,
            dgraph_cluster=dgraph_cluster,
            db=dynamodb_tables,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        AnalyzerDispatcher(
            input_emitter=subgraphs_merged_emitter,
            output_emitter=dispatched_analyzer_emitter,
            analyzers_bucket=analyzers_bucket,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        AnalyzerExecutor(
            input_emitter=dispatched_analyzer_emitter,
            output_emitter=analyzer_matched_emitter,
            dgraph_cluster=dgraph_cluster,
            analyzers_bucket=analyzers_bucket,
            model_plugins_bucket=model_plugins_bucket,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

    EngagementCreator(
        input_emitter=analyzer_matched_emitter,
        network=network,
        forwarder=forwarder,
        dgraph_cluster=dgraph_cluster,
    )

    Provisioner(
        network=network,
        secret=secret,
        db=dynamodb_tables,
        dgraph_cluster=dgraph_cluster,
    )

    OpsAlarms(name="ops-alarms")

    ########################################################################

    # TODO: create everything inside of Api class

    import pulumi_aws as aws

    ux_bucket = Bucket(
        "engagement-ux-bucket",
        website_args=aws.s3.BucketWebsiteArgs(
            index_document="index.html",
        ),
    )
    # Note: This requires `yarn build` to have been run first
    if not LOCAL_GRAPL:
        # Not doing this in Local Grapl at the moment, as we have
        # another means of doing this. We should harmonize this, of
        # course.
        ENGAGEMENT_VIEW_DIR = Path("../src/js/engagement_view/build").resolve()
        ux_bucket.upload_to_bucket(ENGAGEMENT_VIEW_DIR)

    Api(
        network=network,
        secret=secret,
        ux_bucket=ux_bucket,
        db=dynamodb_tables,
        plugins_bucket=model_plugins_bucket,
        forwarder=forwarder,
        dgraph_cluster=dgraph_cluster,
    )

    ########################################################################

    if LOCAL_GRAPL:
        from infra.local import user

        user.local_grapl_user(
            dynamodb_tables.user_auth_table, "grapluser", "graplpassword"
        )


if __name__ == "__main__":
    main()
