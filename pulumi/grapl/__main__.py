import json
import sys
from pathlib import Path
from typing import Set

from typing_extensions import Final

sys.path.insert(0, "..")

import os

import pulumi_aws as aws
import pulumi_nomad as nomad
from infra import config, dynamodb, emitter
from infra.alarms import OpsAlarms

# TODO: temporarily disabled until we can reconnect the ApiGateway to the new
# web UI.
# from infra.api import Api
from infra.api_gateway import ApiGateway
from infra.autotag import register_auto_tags
from infra.bucket import Bucket
from infra.cache import Cache

# TODO: temporarily disabled until we can reconnect the ApiGateway to the new
# web UI.
from infra.kafka import Kafka
from infra.network import Network
from infra.nomad_job import NomadJob, NomadVars
from infra.quiet_docker_build_output import quiet_docker_output

# TODO: temporarily disabled until we can reconnect the ApiGateway to the new
# web UI.
# from infra.secret import JWTSecret, TestUserPassword
from infra.secret import TestUserPassword
from infra.service_queue import ServiceQueue

import pulumi


def _get_subset(inputs: NomadVars, subset: Set[str]) -> NomadVars:
    return {k: inputs[k] for k in subset}


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

    pulumi.export("deployment-name", config.DEPLOYMENT_NAME)
    pulumi.export("test-user-name", config.GRAPL_TEST_USER_NAME)

    # We only set up networking in local since this is handled in a closed project for AWS for our commercial offering
    if config.LOCAL_GRAPL:
        network = Network("grapl-network")

    # TODO: temporarily disabled until we can reconnect the ApiGateway to the new
    # web UI.
    # jwt_secret = JWTSecret()

    test_user_password = TestUserPassword()

    dynamodb_tables = dynamodb.DynamoDB()

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
        "analyzer-matched-subgraphs-bucket", analyzer_matched_emitter.bucket_name
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

    nomad_inputs: Final[NomadVars] = dict(
        analyzer_bucket=analyzers_bucket.bucket,
        analyzer_dispatched_bucket=dispatched_analyzer_emitter.bucket_name,
        analyzer_dispatcher_queue=analyzer_dispatcher_queue.main_queue_url,
        analyzer_executor_queue=analyzer_executor_queue.main_queue_url,
        analyzer_matched_subgraphs_bucket=analyzer_matched_emitter.bucket_name,
        analyzer_dispatcher_dead_letter_queue=analyzer_dispatcher_queue.dead_letter_queue_url,
        aws_region=aws.get_region().name,
        deployment_name=config.DEPLOYMENT_NAME,
        engagement_creator_queue=engagement_creator_queue.main_queue_url,
        graph_merger_queue=graph_merger_queue.main_queue_url,
        graph_merger_dead_letter_queue=graph_merger_queue.dead_letter_queue_url,
        model_plugins_bucket=model_plugins_bucket.bucket,
        node_identifier_queue=node_identifier_queue.main_queue_url,
        node_identifier_dead_letter_queue=node_identifier_queue.dead_letter_queue_url,
        node_identifier_retry_queue=node_identifier_queue.retry_queue_url,
        osquery_generator_queue=osquery_generator_queue.main_queue_url,
        osquery_generator_dead_letter_queue=osquery_generator_queue.dead_letter_queue_url,
        schema_properties_table_name=dynamodb_tables.schema_properties_table.name,
        schema_table_name=dynamodb_tables.schema_table.name,
        session_table_name=dynamodb_tables.dynamic_session_table.name,
        subgraphs_merged_bucket=subgraphs_merged_emitter.bucket_name,
        subgraphs_generated_bucket=subgraphs_generated_emitter.bucket_name,
        sysmon_generator_queue=sysmon_generator_queue.main_queue_url,
        sysmon_generator_dead_letter_queue=sysmon_generator_queue.dead_letter_queue_url,
        test_user_name=config.GRAPL_TEST_USER_NAME,
        unid_subgraphs_generated_bucket=unid_subgraphs_generated_emitter.bucket_name,
        user_auth_table=dynamodb_tables.user_auth_table.name,
        user_session_table=dynamodb_tables.user_session_table.name,
    )

    if config.LOCAL_GRAPL:
        ###################################
        # Local Grapl
        ###################################
        kafka = Kafka("kafka")

        # These are created in `grapl-local-infra.nomad` and not applicable to prod.
        # Nomad will replace the LOCAL_GRAPL_REPLACE_IP sentinel value with the correct IP.
        aws_endpoint = "http://LOCAL_GRAPL_REPLACE_IP:4566"
        kafka_endpoint = "LOCAL_GRAPL_REPLACE_IP:19092"  # intentionally not 29092
        redis_endpoint = "redis://LOCAL_GRAPL_REPLACE_IP:6379"

        pulumi.export("aws-endpoint", aws_endpoint)
        pulumi.export("kafka-endpoint", kafka_endpoint)
        pulumi.export("redis-endpoint", redis_endpoint)

        assert aws.config.access_key
        assert aws.config.secret_key
        grapl_core_job_vars_inputs: Final[NomadVars] = dict(
            # The vars with a leading underscore indicate that the hcl local version of the variable should be used
            # instead of the var version.
            _aws_endpoint=aws_endpoint,
            _redis_endpoint=redis_endpoint,
            aws_access_key_id=aws.config.access_key,
            aws_access_key_secret=aws.config.secret_key,
            rust_log="DEBUG",
        )

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=Path("../../nomad/grapl-core.nomad").resolve(),
            vars=dict(
                **grapl_core_job_vars_inputs,
                **nomad_inputs,
            ),
        )

        nomad_grapl_ingress = NomadJob(
            "grapl-ingress",
            jobspec=Path("../../nomad/grapl-ingress.nomad").resolve(),
            vars={},
        )

        provision_vars = _get_subset(
            dict(
                **grapl_core_job_vars_inputs,
                **nomad_inputs,
            ),
            {
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
            },
        )

        nomad_grapl_provision = NomadJob(
            "grapl-provision",
            jobspec=Path("../../nomad/grapl-provision.nomad").resolve(),
            vars=provision_vars,
            opts=pulumi.ResourceOptions(depends_on=[nomad_grapl_core]),
        )

    else:
        ###################################
        # AWS Grapl
        ###################################
        pulumi_config = pulumi.Config()
        # We use stack outputs from internally developed projects
        # We assume that the stack names will match the grapl stack name
        networking_stack = pulumi.StackReference(
            f"grapl/networking/{pulumi.get_stack()}"
        )
        nomad_server_stack = pulumi.StackReference(f"grapl/nomad/{pulumi.get_stack()}")
        nomad_agents_stack = pulumi.StackReference(
            f"grapl/nomad-agents/{pulumi.get_stack()}"
        )

        vpc_id = networking_stack.require_output("grapl-vpc")
        subnet_ids = networking_stack.require_output("grapl-private-subnet-ids")
        nomad_agent_security_group_id = nomad_agents_stack.require_output(
            "security-group"
        )
        nomad_agent_alb_security_group_id = nomad_agents_stack.require_output(
            "alb-security-group"
        )
        nomad_agent_alb_listener_arn = nomad_agents_stack.require_output(
            "alb-listener-arn"
        )
        nomad_agent_subnet_ids = networking_stack.require_output(
            "nomad-agents-private-subnet-ids"
        )

        cache = Cache(
            "main-cache",
            subnet_ids=subnet_ids,
            vpc_id=vpc_id,
            nomad_agent_security_group_id=nomad_agent_security_group_id,
        )
        artifacts = pulumi_config.require_object("artifacts")

        # Set the nomad address. This can be either set as nomad:address in the config to support ssm port forwarding or
        # taken from the nomad stack
        nomad_config = pulumi.Config("nomad")
        nomad_override_address = nomad_config.get("address")
        # We prefer nomad:address to support overriding in the case of ssm port forwarding
        nomad_address = nomad_override_address or nomad_server_stack.require_output(
            "address"
        )
        nomad_provider = nomad.Provider("nomad-aws", address=nomad_address)

        grapl_core_job_vars: Final[NomadVars] = dict(
            # The vars with a leading underscore indicate that the hcl local version of the variable should be used
            # instead of the var version.
            _redis_endpoint=cache.endpoint,
            container_registry="docker.cloudsmith.io/",
            container_repo="raw/",
            # TODO: consider replacing with the previous per-service `configurable_envvars`
            rust_log="DEBUG",
            # Build Tags. We use per service tags so we can update services independently
            analyzer_dispatcher_tag=artifacts["analyzer-dispatcher"],
            analyzer_executor_tag=artifacts["analyzer-executor"],
            dgraph_tag="latest",
            engagement_creator_tag=artifacts["engagement-creator"],
            graph_merger_tag=artifacts["graph-merger"],
            graphql_endpoint_tag=artifacts["graphql-endpoint"],
            model_plugin_deployer_tag=artifacts["model-plugin-deployer"],
            node_identifier_tag=artifacts["node-identifier"],
            osquery_generator_tag=artifacts["osquery-generator"],
            sysmon_generator_tag=artifacts["sysmon-generator"],
            web_ui_tag=artifacts["grapl-web-ui"],
            **nomad_inputs,
        )

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=Path("../../nomad/grapl-core.nomad").resolve(),
            vars=grapl_core_job_vars,
            opts=pulumi.ResourceOptions(provider=nomad_provider),
        )

        nomad_grapl_ingress = NomadJob(
            "grapl-ingress",
            jobspec=Path("../../nomad/grapl-ingress.nomad").resolve(),
            vars={},
            opts=pulumi.ResourceOptions(provider=nomad_provider),
        )

        grapl_provision_job_vars = _get_subset(
            # IMPORTANT: Any new var added in the dict below also needs to be added to _get_provisioner_job_vars
            dict(
                # The vars with a leading underscore indicate that the hcl local version of the variable should be used
                # instead of the var version.
                container_registry="docker.cloudsmith.io/",
                container_repo="raw/",
                # TODO: consider replacing with the previous per-service `configurable_envvars`
                rust_log="DEBUG",
                provisioner_tag=artifacts["provisioner"],
                **nomad_inputs,
            ),
            {
                "aws_region",
                "container_registry",
                "container_repo",
                "deployment_name",
                "provisioner_tag",
                "rust_log",
                "schema_table_name",
                "schema_properties_table_name",
                "test_user_name",
                "user_auth_table",
            },
        )

        nomad_grapl_provision = NomadJob(
            "grapl-provision",
            jobspec=Path("../../nomad/grapl-provision.nomad").resolve(),
            vars=grapl_provision_job_vars,
            opts=pulumi.ResourceOptions(
                depends_on=[nomad_grapl_core], provider=nomad_provider
            ),
        )

        api_gateway = ApiGateway(
            "grapl-api-gateway",
            nomad_agents_alb_security_group=nomad_agent_alb_security_group_id,
            nomad_agents_alb_listener_arn=nomad_agent_alb_listener_arn,
            nomad_agents_private_subnet_ids=nomad_agent_subnet_ids,
            opts=pulumi.ResourceOptions(
                depends_on=[nomad_grapl_ingress],
            ),
        )
        pulumi.export("stage-url", api_gateway.stage.invoke_url)

    OpsAlarms(name="ops-alarms")


if __name__ == "__main__":
    main()
