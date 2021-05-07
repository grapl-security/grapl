import os

from infra.api import Api
from infra import dynamodb, emitter
from infra.analyzer_dispatcher import AnalyzerDispatcher
from infra.analyzer_executor import AnalyzerExecutor
from infra.api import Api
from infra.autotag import register_auto_tags
from infra.bucket import Bucket
from infra.cache import Cache
from infra.config import DEPLOYMENT_NAME, LOCAL_GRAPL
from infra.dgraph_cluster import DgraphCluster, LocalStandInDgraphCluster
from infra.dgraph_ttl import DGraphTTL
from infra.e2e_test_runner import E2eTestRunner
from infra.engagement_creator import EngagementCreator
from infra.graph_merger import GraphMerger
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.node_identifier import NodeIdentifier
from infra.osquery_generator import OSQueryGenerator
from infra.provision_lambda import Provisioner
from infra.provisioner import Provisioner
from infra.sysmon_generator import SysmonGenerator
from infra.secret import JWTSecret, TestUserPassword
from infra.service_queue import ServiceQueue


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

    test_user_password = TestUserPassword()

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

    # TODO: No _infrastructure_ currently *writes* to this bucket
    analyzers_bucket = Bucket("analyzers-bucket", sse=True)
    model_plugins_bucket = Bucket("model-plugins-bucket", sse=False)

    if LOCAL_GRAPL:
        # We need to create these queues in Local Grapl, because they
        # are otherwise created in the FargateService instances below;
        # we don't run Fargate services in Local Grapl.
        #
        # T_T
        from infra.service_queue import ServiceQueue

        for service in [
            "sysmon-generator",
            "osquery-generator",
            "node-identifier",
            "graph-merger",
            "analyzer-dispatcher",
            "analyzer-executor",
        ]:
            ServiceQueue(service)
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

        E2eTestRunner(network=network)

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
        test_user_password=test_user_password,
    )

    ########################################################################

    # TODO: create everything inside of Api class

    import pulumi_aws as aws

    ux_bucket = Bucket(
        "engagement-ux-bucket",
        website_args=aws.s3.BucketWebsiteArgs(
            index_document="index.html",
        ),
    )
    # TODO: How do we get the *contents* of this bucket uploaded?
    # Max says: "I've introduced a `Bucket.upload_*` function, check it out :)

    Api(
        network=network,
        secret=secret,
        ux_bucket=ux_bucket,
        db=dynamodb_tables,
        plugins_bucket=model_plugins_bucket,
        dgraph_cluster=dgraph_cluster,
    )

if __name__ == "__main__":
    main()
