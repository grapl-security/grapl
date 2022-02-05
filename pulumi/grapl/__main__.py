import sys
from pathlib import Path
from typing import List, Mapping, Set, cast

from pulumi.resource import CustomTimeouts, ResourceOptions
from typing_extensions import Final

sys.path.insert(0, "..")

import pulumi_aws as aws
import pulumi_consul as consul
import pulumi_nomad as nomad
from infra import config, dynamodb, emitter
from infra.alarms import OpsAlarms
from infra.api_gateway import ApiGateway
from infra.autotag import register_auto_tags
from infra.bucket import Bucket
from infra.cache import Cache
from infra.config import AWS_ACCOUNT_ID
from infra.consul_intentions import ConsulIntentions
from infra.docker_images import DockerImageId, DockerImageIdBuilder
from infra.get_hashicorp_provider_address import get_hashicorp_provider_address
from infra.kafka import Kafka
from infra.local.postgres import LocalPostgresInstance
from infra.nomad_job import NomadJob, NomadVars
from infra.postgres import Postgres

# TODO: temporarily disabled until we can reconnect the ApiGateway to the new
# web UI.
# from infra.secret import JWTSecret, TestUserPassword
from infra.secret import TestUserPassword
from infra.service_queue import ServiceQueue

import pulumi


def _get_subset(inputs: NomadVars, subset: Set[str]) -> NomadVars:
    return {k: inputs[k] for k in subset}


def _container_images(
    artifacts: Mapping[str, str], require_artifact: bool = False
) -> Mapping[str, DockerImageId]:
    """
    Build a map of {task name -> docker image identifier}.
    """
    builder = DockerImageIdBuilder(
        container_repository=config.container_repository(),
        artifacts=artifacts,
        require_artifact=require_artifact,
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
        "osquery-generator": builder.build_with_tag("osquery-generator"),
        "plugin-registry": builder.build_with_tag("plugin-registry"),
        "plugin-work-queue": builder.build_with_tag("plugin-work-queue"),
        "provisioner": builder.build_with_tag("provisioner"),
        "sysmon-generator": builder.build_with_tag("sysmon-generator"),
        "web-ui": builder.build_with_tag("grapl-web-ui"),
    }


def subnets_to_single_az(ids: List[str]) -> pulumi.Output[str]:
    subnet_id = ids[-1]
    subnet = aws.ec2.Subnet.get("subnet", subnet_id)
    # for some reason mypy gets hung up on the typing of this
    az: pulumi.Output[str] = subnet.availability_zone
    return az


def main() -> None:
    pulumi_config = pulumi.Config()

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": config.DEPLOYMENT_NAME})

    pulumi.export("deployment-name", config.DEPLOYMENT_NAME)
    pulumi.export("test-user-name", config.GRAPL_TEST_USER_NAME)

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

    plugins_bucket = Bucket("plugins-bucket", sse=True)
    pulumi.export("plugins-bucket", plugins_bucket.bucket)

    plugin_buckets = [
        analyzers_bucket,
        model_plugins_bucket,
    ]

    # These are shared across both local and prod deployments.
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
        plugin_s3_bucket_aws_account_id=AWS_ACCOUNT_ID,
        plugin_s3_bucket_name=plugins_bucket.bucket,
    )

    # To learn more about this syntax, see
    # https://docs.rs/env_logger/0.9.0/env_logger/#enabling-logging
    rust_log_levels = ",".join(
        [
            "DEBUG",
            "h2::codec=WARN",
            "hyper=WARN",
            "rusoto_core=WARN",
            "rustls=WARN",
            "serde_xml_rs=WARN",
        ]
    )
    py_log_level = "DEBUG"

    nomad_grapl_core_timeout = "5m"

    kafka = Kafka(
        "kafka",
        confluent_environment_name=pulumi_config.require("confluent-environment-name"),
    )
    e2e_service_credentials = kafka.service_credentials(service_name="e2e-test-runner")

    pulumi.export("kafka-bootstrap-servers", kafka.bootstrap_servers())
    pulumi.export(
        "kafka-e2e-sasl-username", e2e_service_credentials.apply(lambda c: c.api_key)
    )
    pulumi.export(
        "kafka-e2e-sasl-password", e2e_service_credentials.apply(lambda c: c.api_secret)
    )
    pulumi.export(
        "kafka-e2e-consumer-group-name", kafka.consumer_group("e2e-test-runner")
    )

    if config.LOCAL_GRAPL:
        ###################################
        # Local Grapl
        ###################################

        plugin_registry_db = LocalPostgresInstance(
            name="plugin-registry-db",
            port=5432,
        )

        plugin_work_queue_db = LocalPostgresInstance(
            name="plugin-work-queue-db",
            port=5532,
        )

        pulumi.export("plugin-work-queue-db-hostname", "LOCAL_GRAPL_REPLACE_IP")
        pulumi.export("plugin-work-queue-db-port", str(plugin_work_queue_db.port))
        pulumi.export("plugin-work-queue-db-username", plugin_work_queue_db.username)
        pulumi.export("plugin-work-queue-db-password", plugin_work_queue_db.password)

        # These are created in `grapl-local-infra.nomad` and not applicable to prod.
        # Nomad will replace the LOCAL_GRAPL_REPLACE_IP sentinel value with the correct IP.
        aws_endpoint = "http://LOCAL_GRAPL_REPLACE_IP:4566"
        redis_endpoint = "redis://LOCAL_GRAPL_REPLACE_IP:6379"

        pulumi.export("aws-endpoint", aws_endpoint)
        pulumi.export("redis-endpoint", redis_endpoint)

        aws_config = cast(aws.config.vars._ExportableConfig, aws.config)
        assert aws_config.access_key
        assert aws_config.secret_key

        local_grapl_core_job_vars: Final[NomadVars] = dict(
            # The vars with a leading underscore indicate that the hcl local version of the variable should be used
            # instead of the var version.
            _aws_endpoint=aws_endpoint,
            _redis_endpoint=redis_endpoint,
            aws_access_key_id=aws_config.access_key,
            aws_access_key_secret=aws_config.secret_key,
            container_images=_container_images({}),
            rust_log=rust_log_levels,
            plugin_registry_db_hostname="LOCAL_GRAPL_REPLACE_IP",
            plugin_registry_db_port=str(plugin_registry_db.port),
            plugin_registry_db_username=plugin_registry_db.username,
            plugin_registry_db_password=plugin_registry_db.password,
            plugin_work_queue_db_hostname="LOCAL_GRAPL_REPLACE_IP",
            plugin_work_queue_db_port=str(plugin_work_queue_db.port),
            plugin_work_queue_db_username=plugin_work_queue_db.username,
            plugin_work_queue_db_password=plugin_work_queue_db.password,
            py_log_level=py_log_level,
            **nomad_inputs,
        )

        # This does not use a custom Provider since it will use either a consul:address set in the config or default to
        # http://localhost:8500. This also applies to the NomadJobs defined for LOCAL_GRAPL.
        ConsulIntentions(
            "grapl-core",
            # consul-intentions are stored in the nomad directory so that engineers remember to create/update intentions
            # when they update nomad configs
            intention_directory=Path("../../nomad/consul-intentions").resolve(),
        )

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=Path("../../nomad/grapl-core.nomad").resolve(),
            vars=local_grapl_core_job_vars,
            opts=ResourceOptions(
                custom_timeouts=CustomTimeouts(
                    create=nomad_grapl_core_timeout, update=nomad_grapl_core_timeout
                )
            ),
        )

        nomad_grapl_ingress = NomadJob(
            "grapl-ingress",
            jobspec=Path("../../nomad/grapl-ingress.nomad").resolve(),
            vars={},
        )

        provision_vars = _get_subset(
            local_grapl_core_job_vars,
            {
                "aws_access_key_id",
                "aws_access_key_secret",
                "_aws_endpoint",
                "aws_region",
                "container_images",
                "deployment_name",
                "py_log_level",
                "schema_properties_table_name",
                "schema_table_name",
                "test_user_name",
                "user_auth_table",
            },
        )

        nomad_grapl_provision = NomadJob(
            "grapl-provision",
            jobspec=Path("../../nomad/grapl-provision.nomad").resolve(),
            vars=provision_vars,
            opts=pulumi.ResourceOptions(depends_on=[nomad_grapl_core.job]),
        )

    else:
        ###################################
        # AWS Grapl
        ###################################
        # We use stack outputs from internally developed projects
        # We assume that the stack names will match the grapl stack name
        consul_stack = pulumi.StackReference(f"grapl/consul/{pulumi.get_stack()}")
        networking_stack = pulumi.StackReference(
            f"grapl/networking/{pulumi.get_stack()}"
        )
        nomad_server_stack = pulumi.StackReference(f"grapl/nomad/{pulumi.get_stack()}")
        nomad_agents_stack = pulumi.StackReference(
            f"grapl/nomad-agents/{pulumi.get_stack()}"
        )

        vpc_id = networking_stack.require_output("grapl-vpc")
        subnet_ids = networking_stack.require_output("grapl-private-subnet-ids")
        # Using get_output instead of require_output so that preview passes.
        consul_master_token_secret_id = consul_stack.get_output(
            "consul-master-token-secret-id"
        )
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
        nomad_agent_role = aws.iam.Role.get(
            "nomad-agent-role",
            id=nomad_agents_stack.require_output("iam-role"),
            opts=pulumi.ResourceOptions(parent=nomad_agents_stack),
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

        pulumi.export("plugin-registry-db-hostname", plugin_registry_postgres.host())
        pulumi.export(
            "plugin-registry-db-port", plugin_registry_postgres.port().apply(str)
        )
        pulumi.export(
            "plugin-registry-db-username", plugin_registry_postgres.username()
        )
        pulumi.export(
            "plugin-registry-db-password", plugin_registry_postgres.password()
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

        pulumi.export("kafka-bootstrap-servers", kafka.bootstrap_servers())
        pulumi.export("redis-endpoint", cache.endpoint)

        artifacts = pulumi_config.require_object("artifacts")

        # Set custom provider with the address set
        consul_provider = get_hashicorp_provider_address(
            consul, "consul", consul_stack, {"token": consul_master_token_secret_id}
        )
        nomad_provider = get_hashicorp_provider_address(
            nomad, "nomad", nomad_server_stack
        )

        ConsulIntentions(
            "grapl-core",
            # consul-intentions are stored in the nomad directory so that engineers remember to create/update intentions
            # when they update nomad configs
            intention_directory=Path("../../nomad/consul-intentions").resolve(),
            opts=pulumi.ResourceOptions(provider=consul_provider),
        )

        prod_grapl_core_job_vars: Final[NomadVars] = dict(
            # The vars with a leading underscore indicate that the hcl local version of the variable should be used
            # instead of the var version.
            _redis_endpoint=cache.endpoint,
            container_images=_container_images(artifacts, require_artifact=True),
            plugin_registry_db_hostname=plugin_registry_postgres.host(),
            plugin_registry_db_port=plugin_registry_postgres.port().apply(str),
            plugin_registry_db_username=plugin_registry_postgres.username(),
            plugin_registry_db_password=plugin_registry_postgres.password(),
            plugin_work_queue_db_hostname=plugin_work_queue_postgres.host(),
            plugin_work_queue_db_port=plugin_work_queue_postgres.port().apply(str),
            plugin_work_queue_db_username=plugin_work_queue_postgres.username(),
            plugin_work_queue_db_password=plugin_work_queue_postgres.password(),
            py_log_level=py_log_level,
            rust_log=rust_log_levels,
            **nomad_inputs,
        )

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=Path("../../nomad/grapl-core.nomad").resolve(),
            vars=prod_grapl_core_job_vars,
            opts=pulumi.ResourceOptions(
                provider=nomad_provider,
                custom_timeouts=CustomTimeouts(
                    create=nomad_grapl_core_timeout, update=nomad_grapl_core_timeout
                ),
            ),
        )

        nomad_grapl_ingress = NomadJob(
            "grapl-ingress",
            jobspec=Path("../../nomad/grapl-ingress.nomad").resolve(),
            vars={},
            opts=pulumi.ResourceOptions(provider=nomad_provider),
        )

        grapl_provision_job_vars = _get_subset(
            prod_grapl_core_job_vars,
            {
                "aws_region",
                "container_images",
                "deployment_name",
                "py_log_level",
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
                depends_on=[
                    nomad_grapl_core.job,
                    dynamodb_tables,
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
                # grapl-core contains our dgraph instances
                nomad_grapl_core.urn,
                # We need to re-provision after we start a new dgraph
                nomad_grapl_provision.urn,
                dynamodb_tables.urn,
            ],
        )

    OpsAlarms(name="ops-alarms")


if __name__ == "__main__":
    main()
