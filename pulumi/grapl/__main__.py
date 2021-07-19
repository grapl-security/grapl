import sys

sys.path.insert(0, "..")

import os
from typing import List

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
from infra.e2e_test_runner import E2eTestRunner
from infra.engagement_creator import EngagementCreator
from infra.graph_merger import GraphMerger
from infra.kafka import Kafka
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.node_identifier import NodeIdentifier
from infra.nomad_cluster import NomadCluster
from infra.osquery_generator import OSQueryGenerator
from infra.pipeline_dashboard import PipelineDashboard
from infra.provision_lambda import Provisioner
from infra.quiet_docker_build_output import quiet_docker_output
from infra.secret import JWTSecret, TestUserPassword
from infra.service import ServiceLike
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

    quiet_docker_output()

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": DEPLOYMENT_NAME})

    network = Network("grapl-network")

    dgraph_cluster: DgraphCluster = _create_dgraph_cluster(network=network)

    DGraphTTL(network=network, dgraph_cluster=dgraph_cluster)

    jwt_secret = JWTSecret()

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

    services: List[ServiceLike] = []

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

        kafka = Kafka("kafka")

    else:
        nomad_cluster = NomadCluster(
            "nomad-cluster",
            network=network,
        )

        # No Fargate or Elasticache in Local Grapl
        cache = Cache("main-cache", network=network)

        sysmon_generator = SysmonGenerator(
            input_emitter=sysmon_log_emitter,
            output_emitter=unid_subgraphs_generated_emitter,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        osquery_generator = OSQueryGenerator(
            input_emitter=osquery_log_emitter,
            output_emitter=unid_subgraphs_generated_emitter,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        node_identifier = NodeIdentifier(
            input_emitter=unid_subgraphs_generated_emitter,
            output_emitter=subgraphs_generated_emitter,
            db=dynamodb_tables,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        graph_merger = GraphMerger(
            input_emitter=subgraphs_generated_emitter,
            output_emitter=subgraphs_merged_emitter,
            dgraph_cluster=dgraph_cluster,
            db=dynamodb_tables,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        analyzer_dispatcher = AnalyzerDispatcher(
            input_emitter=subgraphs_merged_emitter,
            output_emitter=dispatched_analyzer_emitter,
            analyzers_bucket=analyzers_bucket,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        analyzer_executor = AnalyzerExecutor(
            input_emitter=dispatched_analyzer_emitter,
            output_emitter=analyzer_matched_emitter,
            dgraph_cluster=dgraph_cluster,
            analyzers_bucket=analyzers_bucket,
            model_plugins_bucket=model_plugins_bucket,
            network=network,
            cache=cache,
            forwarder=forwarder,
        )

        services.extend(
            [
                sysmon_generator,
                osquery_generator,
                node_identifier,
                graph_merger,
                analyzer_dispatcher,
                analyzer_executor,
            ]
        )

    engagement_creator = EngagementCreator(
        input_emitter=analyzer_matched_emitter,
        network=network,
        forwarder=forwarder,
        dgraph_cluster=dgraph_cluster,
    )
    services.append(engagement_creator)

    OpsAlarms(name="ops-alarms")

    PipelineDashboard(services=services)

    ########################################################################

    # TODO: create everything inside of Api class

    import pulumi_aws as aws

    ux_bucket = Bucket(
        "engagement-ux-bucket",
        website_args=aws.s3.BucketWebsiteArgs(
            index_document="index.html",
        ),
    )

    api = Api(
        network=network,
        secret=jwt_secret,
        ux_bucket=ux_bucket,
        db=dynamodb_tables,
        plugins_bucket=model_plugins_bucket,
        forwarder=forwarder,
        dgraph_cluster=dgraph_cluster,
    )

    if not LOCAL_GRAPL:
        from infra.ux import populate_ux_bucket

        # TODO: We are not populating the UX bucket in Local Grapl at
        # the moment, as we have another means of doing this. We
        # should harmonize this, of course.
        populate_ux_bucket(ux_bucket)

        Provisioner(
            network=network,
            test_user_password=test_user_password,
            db=dynamodb_tables,
            dgraph_cluster=dgraph_cluster,
        )

        E2eTestRunner(
            network=network,
            dgraph_cluster=dgraph_cluster,
            api=api,
            jwt_secret=jwt_secret,
            test_user_password=test_user_password,
        )


if __name__ == "__main__":
    main()
