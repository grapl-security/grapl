import sys
from pathlib import Path

sys.path.insert(0, "..")

import os
from typing import Any, List, Mapping

import pulumi_aws as aws
from infra import config, dynamodb, emitter
from infra.alarms import OpsAlarms

# TODO: temporarily disabled until we can reconnect the ApiGateway to the new
# web UI.
# from infra.api import Api
from infra.autotag import register_auto_tags
from infra.bucket import Bucket
from infra.cache import Cache
from infra.dgraph_cluster import DgraphCluster, LocalStandInDgraphCluster
from infra.dgraph_ttl import DGraphTTL

# TODO: temporarily disabled until we can reconnect the ApiGateway to the new
# web UI.
# from infra.e2e_test_runner import E2eTestRunner
from infra.kafka import Kafka
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.nomad_job import NomadJob
from infra.pipeline_dashboard import PipelineDashboard
from infra.quiet_docker_build_output import quiet_docker_output

# TODO: temporarily disabled until we can reconnect the ApiGateway to the new
# web UI.
# from infra.secret import JWTSecret, TestUserPassword
from infra.secret import TestUserPassword
from infra.service import ServiceLike
from infra.service_queue import ServiceQueue

import pulumi


def _create_dgraph_cluster(network: Network) -> DgraphCluster:
    if config.LOCAL_GRAPL:
        return LocalStandInDgraphCluster()
    else:
        return DgraphCluster(
            name=f"{config.DEPLOYMENT_NAME}-dgraph",
            vpc=network.vpc,
        )


def main() -> None:

    if not (config.LOCAL_GRAPL or config.REAL_DEPLOYMENT):
        # Fargate services build their own images and need this
        # variable currently. We don't want this to be checked in
        # Local Grapl, or "real" deployments, though; only developer
        # sandboxes.
        if not os.getenv("DOCKER_BUILDKIT"):
            raise KeyError("Please re-run with 'DOCKER_BUILDKIT=1'")

    quiet_docker_output()

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": config.DEPLOYMENT_NAME})

    network = Network("grapl-network")

    dgraph_cluster: DgraphCluster = _create_dgraph_cluster(network=network)

    DGraphTTL(network=network, dgraph_cluster=dgraph_cluster)

    # TODO: temporarily disabled until we can reconnect the ApiGateway to the new
    # web UI.
    # jwt_secret = JWTSecret()

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
    pulumi.export(
        "analyzer-matched-subgraphs-bucket", analyzer_matched_emitter.bucket.bucket
    )

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

    engagement_creator_queue = ServiceQueue("engagement-creator")
    engagement_creator_queue.subscribe_to_emitter(analyzer_matched_emitter)

    analyzers_bucket = Bucket("analyzers-bucket", sse=True)
    pulumi.export("analyzers-bucket", analyzers_bucket.bucket)
    model_plugins_bucket = Bucket("model-plugins-bucket", sse=False)
    pulumi.export("model-plugins-bucket", model_plugins_bucket.bucket)

    services: List[ServiceLike] = []

    # TODO: Potentially just removable
    ux_bucket = Bucket(
        "engagement-ux-bucket",
        website_args=aws.s3.BucketWebsiteArgs(
            index_document="index.html",
        ),
    )
    pulumi.export("ux-bucket", ux_bucket.bucket)

    nomad_inputs = {
        "analyzer_bucket": analyzers_bucket.bucket,
        "analyzer_dispatched_bucket": dispatched_analyzer_emitter.bucket.bucket,
        "analyzer_dispatcher_queue": analyzer_dispatcher_queue.main_queue_url,
        "analyzer_executor_queue": analyzer_executor_queue.main_queue_url,
        "analyzer_matched_subgraphs_bucket": analyzer_matched_emitter.bucket.bucket,
        "analyzer_dispatcher_dead_letter_queue": analyzer_dispatcher_queue.dead_letter_queue_url,
        "aws_region": aws.get_region().name,
        "deployment_name": config.DEPLOYMENT_NAME,
        "engagement_creator_queue": engagement_creator_queue.main_queue_url,
        "graph_merger_queue": graph_merger_queue.main_queue_url,
        "graph_merger_dead_letter_queue": graph_merger_queue.dead_letter_queue_url,
        "model_plugins_bucket": model_plugins_bucket.bucket,
        "node_identifier_queue": node_identifier_queue.main_queue_url,
        "node_identifier_dead_letter_queue": node_identifier_queue.dead_letter_queue_url,
        "node_identifier_retry_queue": node_identifier_queue.retry_queue_url,
        "osquery_generator_queue": osquery_generator_queue.main_queue_url,
        "osquery_generator_dead_letter_queue": osquery_generator_queue.dead_letter_queue_url,
        "schema_properties_table_name": dynamodb_tables.schema_properties_table.name,
        "schema_table_name": dynamodb_tables.schema_table.name,
        "session_table_name": dynamodb_tables.dynamic_session_table.name,
        "subgraphs_merged_bucket": subgraphs_merged_emitter.bucket,
        "subgraphs_generated_bucket": subgraphs_generated_emitter.bucket,
        "sysmon_generator_queue": sysmon_generator_queue.main_queue_url,
        "sysmon_generator_dead_letter_queue": sysmon_generator_queue.dead_letter_queue_url,
        "test_user_name": config.GRAPL_TEST_USER_NAME,
        "unid_subgraphs_generated_bucket": unid_subgraphs_generated_emitter.bucket,
        "user_auth_table": dynamodb_tables.user_auth_table.name,
        "user_session_table": dynamodb_tables.user_session_table.name,
    }
    pulumi.export("nomad-inputs", nomad_inputs)

    if config.LOCAL_GRAPL:
        kafka = Kafka("kafka")

        # These are created in `grapl-local-infra.nomad` and not applicable to prod.
        # Nomad will replace the LOCAL_GRAPL_REPLACE_IP sentinel value with the correct IP.
        aws_endpoint = "http://LOCAL_GRAPL_REPLACE_IP:4566"
        kafka_endpoint = "LOCAL_GRAPL_REPLACE_IP:19092"  # intentionally not 29092
        redis_endpoint = "redis://LOCAL_GRAPL_REPLACE_IP:6379"

        grapl_core_job_vars_inputs = dict(
            # The vars with a leading underscore indicate that the hcl local version of the variable should be used
            # instead of the var version.
            _aws_endpoint=aws_endpoint,
            _redis_endpoint=redis_endpoint,
            aws_access_key_id=aws.config.access_key,
            aws_access_key_secret=aws.config.secret_key,
            rust_log="DEBUG",
            **nomad_inputs,
        )
        grapl_core_job_vars = pulumi.Output.all(**grapl_core_job_vars_inputs)

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=Path("../../nomad/grapl-core.nomad").resolve(),
            vars=grapl_core_job_vars,
        )

        nomad_grapl_provision = NomadJob(
            "grapl-provision",
            jobspec=Path("../../nomad/grapl-provision.nomad").resolve(),
            vars=pulumi.Output.all(**grapl_core_job_vars_inputs,).apply(
                lambda inputs: {
                    k: inputs[k]
                    for k in {
                        "_aws_endpoint",
                        "aws_access_key_id",
                        "aws_access_key_secret",
                        "aws_region",
                        "deployment_name",
                        "rust_log",
                        "schema_table_name",
                        "schema_properties_table_name",
                        "test_user_name",
                        "user_auth_table",
                    }
                }
            ),
            opts=pulumi.ResourceOptions(depends_on=[nomad_grapl_core]),
        )

        if config.SHOULD_DEPLOY_INTEGRATION_TESTS:

            def _get_integration_test_job_vars(
                inputs: Mapping[str, Any]
            ) -> Mapping[str, Any]:
                return {
                    k: inputs[k]
                    for k in {
                        "_aws_endpoint",
                        "_kafka_endpoint",
                        "_redis_endpoint",
                        "aws_access_key_id",
                        "aws_access_key_secret",
                        "aws_region",
                        "deployment_name",
                        "schema_properties_table_name",
                        "test_user_name",
                    }
                }

            integration_test_job_vars = pulumi.Output.all(
                _kafka_endpoint=kafka_endpoint, **grapl_core_job_vars_inputs
            ).apply(_get_integration_test_job_vars)

            integration_tests = NomadJob(
                "integration-tests",
                jobspec=Path("../../nomad/local/integration-tests.nomad").resolve(),
                vars=integration_test_job_vars,
            )

        if config.SHOULD_DEPLOY_E2E_TESTS:

            def _get_e2e_test_job_vars(inputs: Mapping[str, Any]) -> Mapping[str, Any]:
                return {
                    k: inputs[k]
                    for k in {
                        "_aws_endpoint",
                        "analyzer_bucket",
                        "aws_access_key_id",
                        "aws_access_key_secret",
                        "aws_region",
                        "deployment_name",
                        "schema_properties_table_name",
                        "schema_table_name",
                        "sysmon_generator_queue",
                        "sysmon_log_bucket",
                        "test_user_name",
                    }
                }

            e2e_test_job_vars = pulumi.Output.all(
                sysmon_log_bucket=sysmon_log_emitter.bucket.bucket,
                **grapl_core_job_vars_inputs,
            ).apply(_get_e2e_test_job_vars)
            e2e_tests = NomadJob(
                "e2e-tests",
                jobspec=Path("../../nomad/local/e2e-tests.nomad").resolve(),
                vars=e2e_test_job_vars,
            )

    else:
        cache = Cache("main-cache", network=network)
        pulumi_config = pulumi.Config()
        artifacts = pulumi_config.require_object("artifacts")

        grapl_core_job_aws_vars = pulumi.Output.all(
            # The vars with a leading underscore indicate that the hcl local version of the variable should be used
            # instead of the var version.
            #_redis_endpoint=cache.endpoint,
            container_registry="docker.cloudsmith.io/",
            container_repo="raw/",
            # TODO: consider replacing with the previous per-service `configurable_envvars`
            rust_log="DEBUG",
            # Build Tags. We use per service tags so we can update services independently
            analyzer_dispatcher_tag=artifacts["analyzer-dispatcher"],
            analyzer_executor_tag=artifacts["analyzer-executor"],
            dgraph_tag="latest",
            graph_merger_tag=artifacts["graph-merger"],
            graphql_endpoint_tag=artifacts["graphql-endpoint"],
            node_identifier_tag=artifacts["node-identifier"],
            sysmon_generator_tag=artifacts["sysmon-generator"],
            **nomad_inputs,
        )
        pulumi.export("grapl-core-inputs", grapl_core_job_aws_vars)

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=Path("../../nomad/grapl-core.nomad").resolve(),
            vars=grapl_core_job_aws_vars,
        )

        def _get_provisioner_job_vars(inputs: Mapping[str, Any]) -> Mapping[str, Any]:
            return {
                k: inputs[k]
                for k in {
                    "aws_region",
                    "container_registry",
                    "container_repo",
                    "deployment_name",
                    "rust_log",
                    "schema_table_name",
                    "schema_properties_table_name",
                    "test_user_name",
                    "user_auth_table",
                }
            }

        grapl_provision_job_vars = pulumi.Output.all(
            # The vars with a leading underscore indicate that the hcl local version of the variable should be used
            # instead of the var version.
            container_registry="docker.cloudsmith.io/",
            container_repo="raw/",
            # TODO: consider replacing with the previous per-service `configurable_envvars`
            rust_log="DEBUG",
            provisioner_tag=artifacts["provisioner"],
            **nomad_inputs,
        ).apply(_get_provisioner_job_vars)
        pulumi.export("provisioner_inputs", grapl_provision_job_vars)

        nomad_grapl_provision = NomadJob(
            "grapl-provision",
            jobspec=Path("../../nomad/grapl-provision.nomad").resolve(),
            vars=grapl_provision_job_vars,
            opts=pulumi.ResourceOptions(depends_on=[nomad_grapl_core]),
        )

    OpsAlarms(name="ops-alarms")

    PipelineDashboard(services=services)


if __name__ == "__main__":
    main()
