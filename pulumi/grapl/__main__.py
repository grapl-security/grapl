import sys

sys.path.insert(0, "..")

from typing import List, Mapping, Optional, Set, cast

import pulumi_aws as aws
from infra import config, dynamodb, emitter
from infra.alarms import OpsAlarms
from infra.api_gateway import ApiGateway
from infra.artifacts import ArtifactGetter
from infra.autotag import register_auto_tags
from infra.bucket import Bucket
from infra.cache import Cache
from infra.consul_config import ConsulConfig
from infra.consul_intentions import ConsulIntentions
from infra.consul_service_default import ConsulServiceDefault
from infra.docker_images import DockerImageId, DockerImageIdBuilder
from infra.firecracker_assets import FirecrackerAssets, FirecrackerS3BucketObjects
from infra.hashicorp_provider import (
    get_consul_provider_address,
    get_nomad_provider_address,
)
from infra.kafka import Kafka
from infra.local.postgres import LocalPostgresInstance
from infra.nomad_job import NomadJob, NomadVars
from infra.path import path_from_root
from infra.postgres import Postgres

# TODO: temporarily disabled until we can reconnect the ApiGateway to the new
# web UI.
# from infra.secret import JWTSecret, TestUserPassword
from infra.secret import TestUserPassword
from infra.service_queue import ServiceQueue
from infra.upstream_stacks import UpstreamStacks
from pulumi.resource import CustomTimeouts, ResourceOptions
from typing_extensions import Final

import pulumi


def _get_subset(inputs: NomadVars, subset: Set[str]) -> NomadVars:
    return {k: inputs[k] for k in subset}


def _container_images(artifacts: ArtifactGetter) -> Mapping[str, DockerImageId]:
    """
    Build a map of {task name -> docker image identifier}.
    """
    builder = DockerImageIdBuilder(
        container_repository=config.container_repository(),
        artifacts=artifacts,
    )

    return {
        "analyzer-dispatcher": builder.build_with_tag("analyzer-dispatcher"),
        "analyzer-executor": builder.build_with_tag("analyzer-executor"),
        "dgraph": DockerImageId("dgraph/dgraph:v21.03.1"),
        "engagement-creator": builder.build_with_tag("engagement-creator"),
        "graph-merger": builder.build_with_tag("graph-merger"),
        "graphql-endpoint": builder.build_with_tag("graphql-endpoint"),
        "model-plugin-deployer": builder.build_with_tag("model-plugin-deployer"),
        "node-identifier": builder.build_with_tag("node-identifier"),
        "node-identifier-retry": builder.build_with_tag("node-identifier-retry"),
        "organization-management": builder.build_with_tag("organization-management"),
        "osquery-generator": builder.build_with_tag("osquery-generator"),
        "pipeline-ingress": builder.build_with_tag("pipeline-ingress"),
        "plugin-bootstrap": builder.build_with_tag("plugin-bootstrap"),
        "plugin-registry": builder.build_with_tag("plugin-registry"),
        "plugin-work-queue": builder.build_with_tag("plugin-work-queue"),
        "provisioner": builder.build_with_tag("provisioner"),
        "sysmon-generator": builder.build_with_tag("sysmon-generator"),
        "web-ui": builder.build_with_tag("grapl-web-ui"),
    }


def _get_aws_env_vars_for_local() -> str:
    if not config.LOCAL_GRAPL:
        return "DUMMY_VAR_FOR_PROD = TRUE"

    aws_config = cast(aws.config.vars._ExportableConfig, aws.config)
    assert aws_config.access_key
    assert aws_config.secret_key

    # This uses the weird Mustache {{}} tags because this interpolation eventually
    # gets passed in to a template{} stanza.
    aws_endpoint = 'http://{{ env "attr.unique.network.ip-address" }}:4566'

    return f"""
        GRAPL_AWS_ENDPOINT          = {aws_endpoint}
        GRAPL_AWS_ACCESS_KEY_ID     = {aws_config.access_key}
        GRAPL_AWS_ACCESS_KEY_SECRET = {aws_config.secret_key}
    """


def subnets_to_single_az(ids: List[str]) -> pulumi.Output[str]:
    subnet_id = ids[-1]
    subnet = aws.ec2.Subnet.get("subnet", subnet_id)
    # for some reason mypy gets hung up on the typing of this
    az: pulumi.Output[str] = subnet.availability_zone
    return az


def main() -> None:
    pulumi_config = pulumi.Config()
    artifacts = ArtifactGetter.from_config(pulumi_config)

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags(
        {"pulumi:project": pulumi.get_project(), "pulumi:stack": config.STACK_NAME}
    )

    upstream_stacks: Optional[UpstreamStacks] = None
    nomad_provider: Optional[pulumi.ProviderResource] = None
    consul_provider: Optional[pulumi.ProviderResource] = None
    if not config.LOCAL_GRAPL:
        upstream_stacks = UpstreamStacks()
        nomad_provider = get_nomad_provider_address(upstream_stacks.nomad_server)
        # Using get_output instead of require_output so that preview passes.
        # NOTE wimax Feb 2022: Not sure the above is still the case
        consul_master_token_secret_id = upstream_stacks.consul.get_output(
            "consul-master-token-secret-id"
        )
        consul_provider = get_consul_provider_address(
            upstream_stacks.consul, {"token": consul_master_token_secret_id}
        )

    pulumi.export("test-user-name", config.GRAPL_TEST_USER_NAME)
    test_user_password = TestUserPassword()
    pulumi.export("test-user-password-secret-id", test_user_password.secret_id)

    # TODO: temporarily disabled until we can reconnect the ApiGateway to the new
    # web UI.
    # jwt_secret = JWTSecret()

    dynamodb_tables = dynamodb.DynamoDB()

    kafka = Kafka(
        "kafka",
        confluent_environment_name=pulumi_config.require("confluent-environment-name"),
    )

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

    all_emitters = [
        sysmon_log_emitter,
        osquery_log_emitter,
        unid_subgraphs_generated_emitter,
        subgraphs_generated_emitter,
        subgraphs_merged_emitter,
        dispatched_analyzer_emitter,
        analyzer_matched_emitter,
    ]

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

    pipeline_ingress_healthcheck_polling_interval_ms = "5000"
    pulumi.export(
        "pipeline-ingress-healthcheck-polling-interval-ms",
        pipeline_ingress_healthcheck_polling_interval_ms,
    )
    # This is a hack, because pipeline-ingress is not a kafka consumer, only a
    # producer. The only consumers of this topic are integration tests. That is
    # why we don't look up the consumer group name in the ccloud-bootstrap
    # stack outputs (because it's absent).
    pipeline_ingress_kafka_consumer_group_name = "pipeline-ingress-test"
    pulumi.export(
        "pipeline-ingress-kafka-consumer-group-name",
        pipeline_ingress_kafka_consumer_group_name,
    )

    plugins_bucket = Bucket("plugins-bucket", sse=True)
    pulumi.export("plugins-bucket", plugins_bucket.bucket)

    plugin_buckets = [
        analyzers_bucket,
        model_plugins_bucket,
    ]

    firecracker_s3objs = FirecrackerS3BucketObjects(
        "firecracker-s3-bucket-objects",
        plugins_bucket=plugins_bucket,
        firecracker_assets=FirecrackerAssets(
            "firecracker-assets",
            repository_name=config.cloudsmith_repository_name(),
            artifacts=artifacts,
        ),
    )

    # To learn more about this syntax, see
    # https://docs.rs/env_logger/0.9.0/env_logger/#enabling-logging
    rust_log_levels = ",".join(
        [
            "DEBUG",
            "h2=WARN",
            "hyper=WARN",
            "rusoto_core=WARN",
            "rustls=WARN",
        ]
    )
    py_log_level = "DEBUG"

    aws_env_vars_for_local = _get_aws_env_vars_for_local()
    pulumi.export("aws-env-vars-for-local", aws_env_vars_for_local)

    # These are shared across both local and prod deployments.
    nomad_inputs: Final[NomadVars] = dict(
        analyzer_bucket=analyzers_bucket.bucket,
        analyzer_dispatched_bucket=dispatched_analyzer_emitter.bucket_name,
        analyzer_dispatcher_queue=analyzer_dispatcher_queue.main_queue_url,
        analyzer_executor_queue=analyzer_executor_queue.main_queue_url,
        analyzer_matched_subgraphs_bucket=analyzer_matched_emitter.bucket_name,
        analyzer_dispatcher_dead_letter_queue=analyzer_dispatcher_queue.dead_letter_queue_url,
        aws_env_vars_for_local=aws_env_vars_for_local,
        aws_region=aws.get_region().name,
        container_images=_container_images(artifacts),
        engagement_creator_queue=engagement_creator_queue.main_queue_url,
        graph_merger_queue=graph_merger_queue.main_queue_url,
        graph_merger_dead_letter_queue=graph_merger_queue.dead_letter_queue_url,
        kafka_bootstrap_servers=kafka.bootstrap_servers(),
        model_plugins_bucket=model_plugins_bucket.bucket,
        node_identifier_queue=node_identifier_queue.main_queue_url,
        node_identifier_dead_letter_queue=node_identifier_queue.dead_letter_queue_url,
        node_identifier_retry_queue=node_identifier_queue.retry_queue_url,
        osquery_generator_queue=osquery_generator_queue.main_queue_url,
        osquery_generator_dead_letter_queue=osquery_generator_queue.dead_letter_queue_url,
        pipeline_ingress_healthcheck_polling_interval_ms=pipeline_ingress_healthcheck_polling_interval_ms,
        pipeline_ingress_kafka_consumer_group_name=pipeline_ingress_kafka_consumer_group_name,
        py_log_level=py_log_level,
        rust_log=rust_log_levels,
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
        plugin_registry_kernel_artifact_url=firecracker_s3objs.kernel_s3obj_url,
        plugin_registry_rootfs_artifact_url=firecracker_s3objs.rootfs_s3obj_url,
        plugin_s3_bucket_aws_account_id=config.AWS_ACCOUNT_ID,
        plugin_s3_bucket_name=plugins_bucket.bucket,
    )

    provision_vars: Final[NomadVars] = {
        "test_user_password_secret_id": test_user_password.secret_id,
        **_get_subset(
            nomad_inputs,
            {
                "aws_env_vars_for_local",
                "aws_region",
                "container_images",
                "py_log_level",
                "schema_properties_table_name",
                "schema_table_name",
                "test_user_name",
                "user_auth_table",
            },
        ),
    }

    nomad_grapl_core_timeout = "5m"

    pulumi.export("kafka-bootstrap-servers", kafka.bootstrap_servers())

    e2e_service_credentials = kafka.service_credentials(service_name="e2e-test-runner")

    pulumi.export(
        "kafka-e2e-sasl-username", e2e_service_credentials.apply(lambda c: c.api_key)
    )
    pulumi.export(
        "kafka-e2e-sasl-password", e2e_service_credentials.apply(lambda c: c.api_secret)
    )
    pulumi.export(
        "kafka-e2e-consumer-group-name", kafka.consumer_group("e2e-test-runner")
    )

    pipeline_ingress_kafka_credentials = kafka.service_credentials("pipeline-ingress")

    pulumi.export(
        "pipeline-ingress-kafka-sasl-username",
        pipeline_ingress_kafka_credentials.api_key,
    )
    pulumi.export(
        "pipeline-ingress-kafka-sasl-password",
        pipeline_ingress_kafka_credentials.api_secret,
    )

    ConsulIntentions(
        "consul-intentions",
        # consul-intentions are stored in the nomad directory so that engineers remember to create/update intentions
        # when they update nomad configs
        intention_directory=path_from_root("nomad/consul-intentions").resolve(),
        opts=pulumi.ResourceOptions(provider=consul_provider),
    )

    # Set the protocol explicitly
    consul_web_ui_defaults = ConsulServiceDefault(
        "web-ui",
        service_name="web-ui",
        protocol="http",
        opts=pulumi.ResourceOptions(provider=consul_provider),
    )

    ConsulServiceDefault(
        "graphql-endpoint",
        service_name="graphql-endpoint",
        protocol="http",
        opts=pulumi.ResourceOptions(provider=consul_provider),
    )

    nomad_grapl_ingress = NomadJob(
        "grapl-ingress",
        jobspec=path_from_root("nomad/grapl-ingress.nomad").resolve(),
        vars={},
        opts=pulumi.ResourceOptions(
            provider=nomad_provider,
            # This dependson ensures we've switched the web-ui protocol to http instead of tcp prior. Otherwise there's
            # a protocol mismatch error
            depends_on=[consul_web_ui_defaults],
        ),
    )

    if config.LOCAL_GRAPL:
        ###################################
        # Local Grapl
        ###################################

        # NOTE: The ports for these `LocalPostgresInstance` databases
        # must match what's in `grapl-local-infra.nomad`. That Nomad
        # job will be run _before_ this Pulumi project (because it
        # brings up infrastructure this project depends on in the
        # local case).
        #
        # There's not really a great way to deal with this duplication
        # at the moment, sadly.
        organization_management_db = LocalPostgresInstance(
            name="organization-management-db",
            port=5632,
        )

        plugin_registry_db = LocalPostgresInstance(
            name="plugin-registry-db",
            port=5432,
        )

        plugin_work_queue_db = LocalPostgresInstance(
            name="plugin-work-queue-db",
            port=5532,
        )

        pulumi.export("plugin-work-queue-db-hostname", plugin_work_queue_db.hostname)
        pulumi.export("plugin-work-queue-db-port", str(plugin_work_queue_db.port))
        pulumi.export("plugin-work-queue-db-username", plugin_work_queue_db.username)
        pulumi.export("plugin-work-queue-db-password", plugin_work_queue_db.password)

        # TODO: ADD EXPORTS FOR PLUGIN-REGISTRY

        pulumi.export(
            "organization-management-db-hostname", organization_management_db.hostname
        )
        pulumi.export(
            "organization-management-db-port", str(organization_management_db.port)
        )
        pulumi.export(
            "organization-management-db-username", organization_management_db.username
        )
        pulumi.export(
            "organization-management-db-password", organization_management_db.password
        )

        redis_endpoint = f"redis://{config.HOST_IP_IN_NOMAD}:6379"

        pulumi.export("redis-endpoint", redis_endpoint)

        # Since we're using an IP for Jaeger, this should only be created for local grapl.
        # Once we're using dns addresses we can create it for everything
        ConsulConfig(
            "grapl-core",
            tracing_endpoint="jaeger-zipkin.service.consul",
            opts=pulumi.ResourceOptions(provider=consul_provider),
        )

        local_grapl_core_vars: Final[NomadVars] = dict(
            organization_management_db_hostname=organization_management_db.hostname,
            organization_management_db_port=str(organization_management_db.port),
            organization_management_db_username=organization_management_db.username,
            organization_management_db_password=organization_management_db.password,
            pipeline_ingress_kafka_sasl_username="fake",
            pipeline_ingress_kafka_sasl_password="fake",
            plugin_registry_db_hostname=plugin_registry_db.hostname,
            plugin_registry_db_port=str(plugin_registry_db.port),
            plugin_registry_db_username=plugin_registry_db.username,
            plugin_registry_db_password=plugin_registry_db.password,
            plugin_work_queue_db_hostname=plugin_work_queue_db.hostname,
            plugin_work_queue_db_port=str(plugin_work_queue_db.port),
            plugin_work_queue_db_username=plugin_work_queue_db.username,
            plugin_work_queue_db_password=plugin_work_queue_db.password,
            redis_endpoint=redis_endpoint,
            **nomad_inputs,
        )

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=path_from_root("nomad/grapl-core.nomad").resolve(),
            vars=local_grapl_core_vars,
            opts=ResourceOptions(
                custom_timeouts=CustomTimeouts(
                    create=nomad_grapl_core_timeout, update=nomad_grapl_core_timeout
                )
            ),
        )

        nomad_grapl_provision = NomadJob(
            "grapl-provision",
            jobspec=path_from_root("nomad/grapl-provision.nomad").resolve(),
            vars=provision_vars,
            opts=pulumi.ResourceOptions(depends_on=[nomad_grapl_core.job]),
        )

    else:
        ###################################
        # AWS Grapl
        ###################################
        # We use stack outputs from internally developed projects
        # We assume that the stack names will match the grapl stack name
        assert upstream_stacks, "Upstream stacks previously initialized"

        vpc_id = upstream_stacks.networking.require_output("grapl-vpc")
        subnet_ids = upstream_stacks.networking.require_output(
            "grapl-private-subnet-ids"
        )
        nomad_agent_security_group_id = upstream_stacks.nomad_agents.require_output(
            "security-group"
        )
        nomad_agent_alb_security_group_id = upstream_stacks.nomad_agents.require_output(
            "alb-security-group"
        )
        nomad_agent_alb_listener_arn = upstream_stacks.nomad_agents.require_output(
            "alb-listener-arn"
        )
        nomad_agent_subnet_ids = upstream_stacks.networking.require_output(
            "nomad-agents-private-subnet-ids"
        )
        nomad_agent_role = aws.iam.Role.get(
            "nomad-agent-role",
            id=upstream_stacks.nomad_agents.require_output("iam-role"),
            # NOTE: It's somewhat odd to set a StackReference as a parent
            opts=pulumi.ResourceOptions(parent=upstream_stacks.nomad_agents),
        )

        availability_zone: pulumi.Output[str] = pulumi.Output.from_input(
            subnet_ids
        ).apply(subnets_to_single_az)

        for _bucket in plugin_buckets:
            _bucket.grant_put_permission_to(nomad_agent_role)
            # Analyzer Dispatcher needs to be able to ListObjects on Analyzers
            # Analyzer Executor needs to be able to ListObjects on Model Plugins
            _bucket.grant_get_and_list_to(nomad_agent_role)
        for _emitter in all_emitters:
            _emitter.grant_write_to(nomad_agent_role)
            _emitter.grant_read_to(nomad_agent_role)

        cache = Cache(
            "main-cache",
            subnet_ids=subnet_ids,
            vpc_id=vpc_id,
            nomad_agent_security_group_id=nomad_agent_security_group_id,
        )

        organization_management_postgres = Postgres(
            name="organization-management",
            subnet_ids=subnet_ids,
            vpc_id=vpc_id,
            availability_zone=availability_zone,
            nomad_agent_security_group_id=nomad_agent_security_group_id,
        )

        plugin_registry_postgres = Postgres(
            name="plugin-registry",
            subnet_ids=subnet_ids,
            vpc_id=vpc_id,
            availability_zone=availability_zone,
            nomad_agent_security_group_id=nomad_agent_security_group_id,
        )

        plugin_work_queue_postgres = Postgres(
            name="plugin-work-queue",
            subnet_ids=subnet_ids,
            vpc_id=vpc_id,
            availability_zone=availability_zone,
            nomad_agent_security_group_id=nomad_agent_security_group_id,
        )

        pulumi.export(
            "organization-management-db-hostname",
            organization_management_postgres.host(),
        )
        pulumi.export(
            "organization-management-db-port",
            organization_management_postgres.port().apply(str),
        )
        pulumi.export(
            "organization-management-db-username",
            organization_management_postgres.username(),
        )
        pulumi.export(
            "organization-management-db-password",
            organization_management_postgres.password(),
        )

        pulumi.export(
            "plugin-work-queue-db-hostname", plugin_work_queue_postgres.host()
        )
        pulumi.export(
            "plugin-work-queue-db-port", plugin_work_queue_postgres.port().apply(str)
        )
        pulumi.export(
            "plugin-work-queue-db-username",
            plugin_work_queue_postgres.username(),
        )
        pulumi.export(
            "plugin-work-queue-db-password",
            plugin_work_queue_postgres.password(),
        )

        pulumi.export("redis-endpoint", cache.endpoint)

        prod_grapl_core_vars: Final[NomadVars] = dict(
            # The vars with a leading underscore indicate that the hcl local version of the variable should be used
            # instead of the var version.
            organization_management_db_hostname=organization_management_postgres.host(),
            organization_management_db_port=organization_management_postgres.port().apply(
                str
            ),
            organization_management_db_username=organization_management_postgres.username(),
            organization_management_db_password=organization_management_postgres.password(),
            pipeline_ingress_kafka_sasl_username=pipeline_ingress_kafka_credentials.api_key,
            pipeline_ingress_kafka_sasl_password=pipeline_ingress_kafka_credentials.api_secret,
            plugin_registry_db_hostname=plugin_registry_postgres.host(),
            plugin_registry_db_port=plugin_registry_postgres.port().apply(str),
            plugin_registry_db_username=plugin_registry_postgres.username(),
            plugin_registry_db_password=plugin_registry_postgres.password(),
            plugin_work_queue_db_hostname=plugin_work_queue_postgres.host(),
            plugin_work_queue_db_port=plugin_work_queue_postgres.port().apply(str),
            plugin_work_queue_db_username=plugin_work_queue_postgres.username(),
            plugin_work_queue_db_password=plugin_work_queue_postgres.password(),
            redis_endpoint=cache.endpoint,
            **nomad_inputs,
        )

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=path_from_root("nomad/grapl-core.nomad").resolve(),
            vars=prod_grapl_core_vars,
            opts=pulumi.ResourceOptions(
                provider=nomad_provider,
                custom_timeouts=CustomTimeouts(
                    create=nomad_grapl_core_timeout, update=nomad_grapl_core_timeout
                ),
            ),
        )

        nomad_grapl_provision = NomadJob(
            "grapl-provision",
            jobspec=path_from_root("nomad/grapl-provision.nomad").resolve(),
            vars=provision_vars,
            opts=pulumi.ResourceOptions(
                depends_on=[
                    nomad_grapl_core.job,
                ],
                provider=nomad_provider,
            ),
        )

        api_gateway = ApiGateway(
            "grapl-api-gateway",
            nomad_agents_alb_security_group=nomad_agent_alb_security_group_id,
            nomad_agents_alb_listener_arn=nomad_agent_alb_listener_arn,
            nomad_agents_private_subnet_ids=nomad_agent_subnet_ids,
            opts=pulumi.ResourceOptions(
                depends_on=[nomad_grapl_ingress.job],
            ),
        )
        pulumi.export("stage-url", api_gateway.stage.invoke_url)

        # Describes resources that should be destroyed/updated between
        # E2E-in-AWS runs.
        pulumi.export(
            "stateful-resource-urns",
            [
                # We need to re-provision
                nomad_grapl_provision.urn,
                dynamodb_tables.urn,
            ],
        )

    OpsAlarms(name="ops-alarms")


if __name__ == "__main__":
    main()
