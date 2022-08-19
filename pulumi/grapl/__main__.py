import sys

sys.path.insert(0, "..")

from typing import Mapping, cast

import pulumi_aws as aws
from infra import config, dynamodb, log_levels
from infra.alarms import OpsAlarms
from infra.api_gateway import ApiGateway
from infra.artifacts import ArtifactGetter
from infra.autotag import register_auto_tags
from infra.bucket import Bucket
from infra.config import repository_path
from infra.consul_config import ConsulConfig
from infra.consul_intentions import ConsulIntentions
from infra.consul_service_default import ConsulServiceDefault
from infra.docker_images import DockerImageId, DockerImageIdBuilder
from infra.firecracker_assets import (
    FirecrackerAssets,
    FirecrackerS3BucketObjects,
    FirecrackerS3BucketObjectsProtocol,
    MockFirecrackerS3BucketObjects,
)
from infra.hashicorp_provider import (
    get_consul_provider_address,
    get_nomad_provider_address,
)
from infra.kafka import Credential, Kafka
from infra.local.postgres import LocalPostgresInstance
from infra.local.scylla import LocalScyllaInstance, NomadServiceScyllaResource
from infra.nomad_job import NomadJob, NomadVars
from infra.nomad_service_postgres import NomadServicePostgresResource
from infra.observability_env_vars import observability_env_vars_for_local
from infra.postgres import Postgres
from infra.scylla import ProdScyllaInstance

# TODO: temporarily disabled until we can reconnect the ApiGateway to the new
# web UI.
# from infra.secret import JWTSecret, TestUserPassword
from infra.secret import TestUserPassword
from infra.upstream_stacks import UpstreamStacks
from pulumi.resource import CustomTimeouts, ResourceOptions
from typing_extensions import Final

import pulumi

"""
https://github.com/grapl-security/issue-tracker/issues/908
This can eventually be removed once we remove HaxDocker in favor of Firecracker
"""
USE_HAX_DOCKER_RUNTIME: bool = True


def _get_subset(inputs: NomadVars, subset: set[str]) -> NomadVars:
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
        "analyzer-execution-sidecar": DockerImageId("TODO implement analzyer executor"),
        "dgraph": DockerImageId("dgraph/dgraph:v21.03.1"),
        "event-source": builder.build_with_tag("event-source"),
        "generator-dispatcher": builder.build_with_tag("generator-dispatcher"),
        "generator-execution-sidecar": builder.build_with_tag(
            "generator-execution-sidecar"
        ),
        "graph-merger": builder.build_with_tag("graph-merger"),
        "graph-query-service": builder.build_with_tag("graph-query-service"),
        "graphql-endpoint": builder.build_with_tag("graphql-endpoint"),
        "hax-docker-plugin-runtime": DockerImageId("debian:bullseye-slim"),
        "kafka-retry": builder.build_with_tag("kafka-retry"),
        "node-identifier": builder.build_with_tag("node-identifier"),
        "organization-management": builder.build_with_tag("organization-management"),
        "pipeline-ingress": builder.build_with_tag("pipeline-ingress"),
        "plugin-bootstrap": builder.build_with_tag("plugin-bootstrap"),
        "plugin-registry": builder.build_with_tag("plugin-registry"),
        "plugin-work-queue": builder.build_with_tag("plugin-work-queue"),
        "provisioner": builder.build_with_tag("provisioner"),
        "graph-schema-manager": builder.build_with_tag("graph-schema-manager"),
        "web-ui": builder.build_with_tag("grapl-web-ui"),
        "uid-allocator": builder.build_with_tag("uid-allocator"),
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


def subnets_to_single_az(ids: list[str]) -> pulumi.Output[str]:
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

    upstream_stacks: UpstreamStacks | None = None
    nomad_provider: pulumi.ProviderResource | None = None
    consul_provider: pulumi.ProviderResource | None = None
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

    plugin_registry_bucket = Bucket("plugin-registry-bucket", sse=True)

    all_plugin_buckets = [
        plugin_registry_bucket,
    ]

    pipeline_ingress_healthcheck_polling_interval_ms = "5000"
    organization_management_healthcheck_polling_interval_ms = "5000"

    firecracker_s3objs: FirecrackerS3BucketObjectsProtocol = (
        MockFirecrackerS3BucketObjects()
        if USE_HAX_DOCKER_RUNTIME
        else FirecrackerS3BucketObjects(
            "firecracker-s3-bucket-objects",
            plugins_bucket=plugin_registry_bucket,
            firecracker_assets=FirecrackerAssets(
                "firecracker-assets",
                repository_name=config.cloudsmith_repository_name(),
                artifacts=artifacts,
            ),
        )
    )

    aws_env_vars_for_local = _get_aws_env_vars_for_local()
    pulumi.export("aws-env-vars-for-local", aws_env_vars_for_local)

    kafka_services = (
        "generator-dispatcher",
        "generator-dispatcher-retry",
        "graph-generator",
        "graph-merger",
        "node-identifier",
        "pipeline-ingress",
        "plugin-work-queue",
    )
    kafka_service_credentials = {
        service: kafka.service_credentials(service).apply(
            Credential.to_nomad_service_creds
        )
        for service in kafka_services
    }
    kafka_consumer_services = (
        "generator-dispatcher",
        "generator-dispatcher-retry",
        "graph-generator",
        "graph-merger",
        "node-identifier",
    )
    kafka_consumer_groups = {
        service: kafka.consumer_group(service) for service in kafka_consumer_services
    }

    observability_env_vars = observability_env_vars_for_local()

    # This Google client ID is used by grapl-web-ui for authenticating users via Sign In With Google.
    # TODO: This should be moved to Pulumi config somehwo, but I'm not sure the best way to do that atm.
    google_client_id = (
        "340240241744-6mu4h5i6h9j7ntp45p3aki81lqd4gc8t.apps.googleusercontent.com"
    )

    # These are shared across both local and prod deployments.
    nomad_inputs: Final[NomadVars] = dict(
        aws_env_vars_for_local=aws_env_vars_for_local,
        aws_region=aws.get_region().name,
        container_images=_container_images(artifacts),
        kafka_bootstrap_servers=kafka.bootstrap_servers(),
        kafka_credentials=kafka_service_credentials,
        kafka_consumer_groups=kafka_consumer_groups,
        observability_env_vars=observability_env_vars,
        organization_management_healthcheck_polling_interval_ms=organization_management_healthcheck_polling_interval_ms,
        pipeline_ingress_healthcheck_polling_interval_ms=pipeline_ingress_healthcheck_polling_interval_ms,
        plugin_registry_bucket_aws_account_id=config.AWS_ACCOUNT_ID,
        plugin_registry_bucket_name=plugin_registry_bucket.bucket,
        plugin_registry_kernel_artifact_url=firecracker_s3objs.kernel_s3obj_url,
        plugin_registry_rootfs_artifact_url=firecracker_s3objs.rootfs_s3obj_url,
        py_log_level=log_levels.PY_LOG_LEVEL,
        rust_log=log_levels.RUST_LOG_LEVELS,
        schema_properties_table_name=dynamodb_tables.schema_properties_table.name,
        schema_table_name=dynamodb_tables.schema_table.name,
        session_table_name=dynamodb_tables.dynamic_session_table.name,
        test_user_name=config.GRAPL_TEST_USER_NAME,
        user_auth_table=dynamodb_tables.user_auth_table.name,
        user_session_table=dynamodb_tables.user_session_table.name,
        google_client_id=google_client_id,
    )

    provision_vars: Final[NomadVars] = {
        "test_user_password_secret_id": test_user_password.secret_id,
        **_get_subset(
            nomad_inputs,
            {
                "aws_env_vars_for_local",
                "aws_region",
                "container_images",
                "observability_env_vars",
                "py_log_level",
                "schema_properties_table_name",
                "schema_table_name",
                "test_user_name",
                "user_auth_table",
            },
        ),
    }

    nomad_grapl_core_timeout = "5m"

    ConsulIntentions(
        "consul-intentions",
        # consul-intentions are stored in the nomad directory so that engineers remember to create/update intentions
        # when they update nomad configs
        intention_directory=repository_path("nomad/consul-intentions"),
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
        jobspec=repository_path("nomad/grapl-ingress.nomad"),
        vars={},
        opts=pulumi.ResourceOptions(
            provider=nomad_provider,
            # This dependson ensures we've switched the web-ui protocol to http instead of tcp prior. Otherwise there's
            # a protocol mismatch error
            depends_on=[consul_web_ui_defaults],
        ),
    )

    organization_management_db: NomadServicePostgresResource
    plugin_registry_db: NomadServicePostgresResource
    plugin_work_queue_db: NomadServicePostgresResource
    uid_allocator_db: NomadServicePostgresResource
    event_source_db: NomadServicePostgresResource
    graph_schema_manager_db: NomadServicePostgresResource

    graph_db: NomadServiceScyllaResource = (
        LocalScyllaInstance(
            name="graph-db",
            port=9042,
        )
        if config.LOCAL_GRAPL
        else ProdScyllaInstance("graph-db")
    )

    if config.LOCAL_GRAPL:
        ###################################
        # Local Grapl
        ###################################

        # NOTE: The ports for these `LocalPostgresInstance` databases must
        # match what's in `grapl-local-infra.nomad`, specifically
        # local { database_descriptors }.
        #
        # That Nomad job will be run _before_ this Pulumi project (because it
        # brings up infrastructure this project depends on in the
        # local case).
        #
        # There's not really a great way to deal with this duplication
        # at the moment, sadly.
        plugin_registry_db = LocalPostgresInstance(
            name="plugin-registry-db",
            port=5432,
        )

        plugin_work_queue_db = LocalPostgresInstance(
            name="plugin-work-queue-db",
            port=5433,
        )

        organization_management_db = LocalPostgresInstance(
            name="organization-management-db",
            port=5434,
        )

        uid_allocator_db = LocalPostgresInstance(
            name="uid-allocator-db",
            port=5435,
        )

        event_source_db = LocalPostgresInstance(
            name="event-source-db",
            port=5436,
        )

        graph_schema_manager_db = LocalPostgresInstance(
            name="graph-schema-manager-db", port=5437
        )

        # Since we're using an IP for Jaeger, this should only be created for local grapl.
        # Once we're using dns addresses we can create it for everything
        ConsulConfig(
            "grapl-core",
            tracing_endpoint="jaeger-zipkin.service.consul",
            opts=pulumi.ResourceOptions(provider=consul_provider),
        )

        local_grapl_core_vars: Final[NomadVars] = dict(
            graph_db=graph_db.to_nomad_scylla_args(),
            event_source_db=event_source_db.to_nomad_service_db_args(),
            organization_management_db=organization_management_db.to_nomad_service_db_args(),
            plugin_registry_db=plugin_registry_db.to_nomad_service_db_args(),
            plugin_work_queue_db=plugin_work_queue_db.to_nomad_service_db_args(),
            graph_schema_manager_db=graph_schema_manager_db.to_nomad_service_db_args(),
            uid_allocator_db=uid_allocator_db.to_nomad_service_db_args(),
            **nomad_inputs,
        )

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=repository_path("nomad/grapl-core.nomad"),
            vars=local_grapl_core_vars,
            opts=ResourceOptions(
                custom_timeouts=CustomTimeouts(
                    create=nomad_grapl_core_timeout, update=nomad_grapl_core_timeout
                )
            ),
        )

        nomad_grapl_provision = NomadJob(
            "grapl-provision",
            jobspec=repository_path("nomad/grapl-provision.nomad"),
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

        for bucket in all_plugin_buckets:
            bucket.grant_put_permission_to(nomad_agent_role)
            bucket.grant_get_and_list_to(nomad_agent_role)

        (
            organization_management_db,
            plugin_registry_db,
            plugin_work_queue_db,
            uid_allocator_db,
            event_source_db,
            graph_schema_manager_db,
        ) = (
            Postgres(
                name=db_resource_name,
                subnet_ids=subnet_ids,
                vpc_id=vpc_id,
                availability_zone=availability_zone,
                nomad_agent_security_group_id=nomad_agent_security_group_id,
            )
            for db_resource_name in (
                "organization-management",
                "plugin-registry",
                "plugin-work-queue",
                "uid-allocator-db",
                "event-source-db",
                "graph-schema-manager-db",
            )
        )

        prod_grapl_core_vars: Final[NomadVars] = dict(
            graph_db=graph_db.to_nomad_scylla_args(),
            event_source_db=event_source_db.to_nomad_service_db_args(),
            organization_management_db=organization_management_db.to_nomad_service_db_args(),
            plugin_registry_db=plugin_registry_db.to_nomad_service_db_args(),
            plugin_work_queue_db=plugin_work_queue_db.to_nomad_service_db_args(),
            graph_schema_manager_db=graph_schema_manager_db.to_nomad_service_db_args(),
            uid_allocator_db=uid_allocator_db.to_nomad_service_db_args(),
            **nomad_inputs,
        )

        # make it easy to debug nomad var issues
        if pulumi.runtime.is_dry_run():
            pulumi.export("prod-grapl-core-vars", prod_grapl_core_vars)

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=repository_path("nomad/grapl-core.nomad"),
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
            jobspec=repository_path("nomad/grapl-provision.nomad"),
            vars=provision_vars,
            opts=pulumi.ResourceOptions(
                depends_on=[
                    nomad_grapl_core.job,
                ],
                provider=nomad_provider,
            ),
        )

        NomadJob(
            "grapl-observability",
            jobspec=repository_path("nomad/observability.nomad"),
            vars={},
            opts=pulumi.ResourceOptions(
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

    pulumi.export(
        "organization-management-db",
        organization_management_db.to_nomad_service_db_args(),
    )

    pulumi.export(
        "plugin-work-queue-db", plugin_work_queue_db.to_nomad_service_db_args()
    )

    pulumi.export("user-auth-table", dynamodb_tables.user_auth_table.name)
    pulumi.export("user-session-table", dynamodb_tables.user_session_table.name)

    pulumi.export("graph-db", graph_db.to_nomad_scylla_args())

    # Not currently imported in integration tests:
    # - uid-allocator-db
    # - plugin-registry-db


if __name__ == "__main__":
    main()
