import sys
from copy import deepcopy
from pathlib import Path
from typing import Any, Mapping, MutableMapping, Optional, Set, cast

from typing_extensions import Final

sys.path.insert(0, "..")

import os

import pulumi_aws as aws
import pulumi_consul as consul
import pulumi_nomad as nomad
import requests
from infra import config, dynamodb, emitter
from infra.alarms import OpsAlarms

# TODO: temporarily disabled until we can reconnect the ApiGateway to the new
# web UI.
# from infra.api import Api
from infra.api_gateway import ApiGateway
from infra.autotag import register_auto_tags
from infra.bucket import Bucket
from infra.cache import Cache
from infra.consul_acl_policies import ConsulAclPolicies
from infra.consul_intentions import ConsulIntentions
from infra.docker_image_tag import version_tag
from infra.get_hashicorp_provider_address import get_hashicorp_provider_address
from infra.grapl_consul_acls import GraplConsulAcls

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
from pulumi.dynamic import (
    CreateResult,
    DiffResult,
    Resource,
    ResourceProvider,
    UpdateResult,
)

import pulumi


def _get_subset(inputs: NomadVars, subset: Set[str]) -> NomadVars:
    return {k: inputs[k] for k in subset}


def grapl_core_docker_image_tags(
    artifacts: Mapping[str, str], require_artifact: bool = False
) -> NomadVars:
    # partial apply some repeated args
    version_tag_alias = lambda key: version_tag(key, artifacts, require_artifact)

    return dict(
        analyzer_dispatcher_tag=version_tag_alias("analyzer-dispatcher"),
        analyzer_executor_tag=version_tag_alias("analyzer-executor"),
        dgraph_tag="latest",
        engagement_creator_tag=version_tag_alias("engagement-creator"),
        graph_merger_tag=version_tag_alias("graph-merger"),
        graphql_endpoint_tag=version_tag_alias("graphql-endpoint"),
        model_plugin_deployer_tag=version_tag_alias("model-plugin-deployer"),
        node_identifier_tag=version_tag_alias("node-identifier"),
        osquery_generator_tag=version_tag_alias("osquery-generator"),
        sysmon_generator_tag=version_tag_alias("sysmon-generator"),
        web_ui_tag=version_tag_alias("grapl-web-ui"),
    )


# Dynamic Resource Provider Notes
########################################################################
# Dynamic resource providers in Python apparently *must* be defined in
# the __main__.py file due to problems in serializing the
# implementations.
#
# Additionally, despite the documented suggestion to define resource
# inputs using Python classes, this does not appear to work (again,
# due to serialization issues). Instead, we can use plain Python
# dictionaries. Not ideal, but it has the benefit of actually working.
class ConsulAclBootstrap(Resource):
    id: pulumi.Output[str]
    consul_address: pulumi.Output[str]
    secret_token: pulumi.Output[str]

    def __init__(
        self,
        name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ):

        consul_config = pulumi.Config("consul")
        super().__init__(
            ConsulAclBootstrapProvider(),
            name,
            {
                "consul_address": consul_config.require("address"),
                "secret_token": consul_config.get("token"),
            },
            opts,
        )


class ConsulAclBootstrapProvider(ResourceProvider):
    # The consul acl bootstrapping can only be run once, after that it returns a 403.
    # If this is running on a previously bootstrapped cluster, we require a token be configured and create a fake id
    def create(self, inputs: Mapping[str, Any]) -> CreateResult:
        response = requests.put(f"{inputs['consul_address']}/v1/acl/bootstrap")
        if response.status_code == requests.codes.ok:
            bootstrap_id = response.json()["AccessorID"]
            secret_id = response.json()["SecretID"]
        elif response.status_code == requests.codes.forbidden:
            # We've already run the bootstrap process so instead let's grab the token from the config
            if inputs["secret_token"] is not None:
                secret_id = inputs["secret_token"]
            else:
                raise Exception(
                    "If Consul ACL Bootstrapping is complete you must set consul:token with the token"
                )
            # Create a fake static id since we don't have the accessor_id available
            bootstrap_id = "fake-id-since-previously-bootstrapped"
        else:
            response.raise_for_status()

        # As is customary in Pulumi, all inputs are available as
        # outputs (this must include our injected Vault address and
        # namespace values, which will be required when we try to
        # delete the resource).
        #
        outs = cast(MutableMapping[str, Any], deepcopy(inputs))
        # ... plus the secret token
        outs["id"] = bootstrap_id
        outs["secret_token"] = secret_id
        return CreateResult(id_=bootstrap_id, outs=outs)

    def delete(self, id: str, props: Mapping[str, Any]) -> None:
        pass

    # The function that determines if an existing resource whose inputs were
    # modified needs to be updated or entirely replaced
    def diff(
        self, id: str, old_inputs: Mapping[str, Any], new_inputs: Mapping[str, Any]
    ) -> DiffResult:
        """
        This is a no-op since consul acl bootstrapping can only be run once
        """
        return DiffResult(
            # If the old and new inputs don't match, the resource needs to be updated/replaced
            changes=old_inputs != new_inputs,
            # If the replaces[] list is empty, nothing important was changed, and we do not have to
            # replace the resource.
            replaces=[],
            # An optional list of inputs that are always constant
            stables=None,
            # The existing resource is deleted before the new one is created
            delete_before_replace=True,
        )

    def update(
        self, _id: str, olds: Mapping[str, Any], _news: Mapping[str, Any]
    ) -> UpdateResult:
        """
        This is a no-op, but required to get around https://github.com/pulumi/pulumi/issues/7809.
        """
        return UpdateResult(outs=olds)


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

        # This does not use a custom Provider since it will use either a consul:address set in the config or default to
        # http://localhost:8500. This also applies to the NomadJobs defined for LOCAL_GRAPL.
        bootstrap = ConsulAclBootstrap(
            name="consul-acl-bootstrap",
            opts=pulumi.ResourceOptions(additional_secret_outputs=["secret_token"]),
        )
        pulumi.export("bootstrap", bootstrap)
        pulumi.export("bootstrapid", bootstrap.id)

        consul_provider = consul.Provider(
            "consul",
            address=pulumi.Config("consul").get("address"),
            token=bootstrap.secret_token,
        )

        consul_acl_policies = ConsulAclPolicies(
            "grapl",
            acl_directory=Path("../consul-acl-policies").resolve(),
            opts=pulumi.ResourceOptions(provider=consul_provider),
        )

        grapl_acls = GraplConsulAcls(
            "grapl",
            policies=consul_acl_policies.policies,
            opts=pulumi.ResourceOptions(provider=consul_provider),
        )
        pulumi.export("ui_read_only_token", grapl_acls.ui_read_only_token.id)
        pulumi.export("ui_read_write_token", grapl_acls.ui_read_write_token.id)
        pulumi.export(
            "default_consul_agent_token", grapl_acls.default_consul_agent_token.id
        )

        ConsulIntentions(
            "grapl-core",
            # consul-intentions are stored in the nomad directory so that engineers remember to create/update intentions
            # when they update nomad configs
            intention_directory=Path("../../nomad/consul-intentions").resolve(),
            opts=pulumi.ResourceOptions(
                depends_on=grapl_acls, provider=consul_provider
            ),
        )

        nomad_grapl_core = NomadJob(
            "grapl-core",
            jobspec=Path("../../nomad/grapl-core.nomad").resolve(),
            vars=dict(
                **grapl_core_job_vars_inputs,
                **nomad_inputs,
                **grapl_core_docker_image_tags({}),
            ),
        )

        nomad_grapl_ingress = NomadJob(
            "grapl-ingress",
            jobspec=Path("../../nomad/grapl-ingress.nomad").resolve(),
            vars={},
        )

        provision_vars = _get_subset(
            dict(
                provisioner_tag=version_tag("provisioner", {}, require_artifact=False),
                **grapl_core_job_vars_inputs,
                **nomad_inputs,
            ),
            {
                "aws_access_key_id",
                "aws_access_key_secret",
                "_aws_endpoint",
                "aws_region",
                "deployment_name",
                "provisioner_tag",
                "rust_log",
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
            opts=pulumi.ResourceOptions(depends_on=[nomad_grapl_core]),
        )

    else:
        ###################################
        # AWS Grapl
        ###################################
        pulumi_config = pulumi.Config()
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

        pulumi.export("kafka-endpoint", "dummy_value_while_we_wait_for_kafka")
        pulumi.export("redis-endpoint", cache.endpoint)

        artifacts = pulumi_config.require_object("artifacts")

        # Set custom provider with the address set
        nomad_provider = get_hashicorp_provider_address(
            nomad, "nomad", nomad_server_stack
        )

        bootstrap = ConsulAclBootstrap(
            "consul-acl-bootstrap",
            pulumi.ResourceOptions(additional_secret_outputs=["secret_token"]),
        )

        consul_provider_with_token = get_hashicorp_provider_address(
            consul, "consul", consul_stack, {"token": bootstrap.secret_token}
        )

        consul_acl_policies = ConsulAclPolicies(
            "grapl",
            acl_directory=Path("../consul-acl-policies").resolve(),
            opts=pulumi.ResourceOptions(provider=consul_provider_with_token),
        )

        grapl_acls = GraplConsulAcls(
            "grapl",
            policies=consul_acl_policies.policies,
            opts=pulumi.ResourceOptions(provider=consul_provider_with_token),
        )
        pulumi.export("ui_read_only_token", grapl_acls.ui_read_only_token.id)
        pulumi.export("ui_read_write_token", grapl_acls.ui_read_write_token.id)
        pulumi.export(
            "default_consul_agent_token", grapl_acls.default_consul_agent_token.id
        )

        ConsulIntentions(
            "grapl-core",
            # consul-intentions are stored in the nomad directory so that engineers remember to create/update intentions
            # when they update nomad configs
            intention_directory=Path("../../nomad/consul-intentions").resolve(),
            opts=pulumi.ResourceOptions(
                provider=consul_provider_with_token, depends_on=[grapl_acls]
            ),
        )

        grapl_core_job_vars: Final[NomadVars] = dict(
            # The vars with a leading underscore indicate that the hcl local version of the variable should be used
            # instead of the var version.
            _redis_endpoint=cache.endpoint,
            container_registry="docker.cloudsmith.io/",
            container_repo="raw/",
            # TODO: consider replacing with the previous per-service `configurable_envvars`
            rust_log="DEBUG",
            **nomad_inputs,
            # Build Tags. We use per service tags so we can update services independently
            **grapl_core_docker_image_tags(artifacts, require_artifact=True),
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
            dict(
                # The vars with a leading underscore indicate that the hcl local version of the variable should be used
                # instead of the var version.
                container_registry="docker.cloudsmith.io/",
                container_repo="raw/",
                # TODO: consider replacing with the previous per-service `configurable_envvars`
                rust_log="DEBUG",
                provisioner_tag=version_tag(
                    "provisioner", artifacts, require_artifact=True
                ),
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
