import json
from typing import List, Mapping, Optional, Sequence, Tuple, Union, cast

import pulumi_aws as aws
import pulumi_docker as docker
from infra.cache import Cache
from infra.config import (
    DEPLOYMENT_NAME,
    REAL_DEPLOYMENT,
    SERVICE_LOG_RETENTION_DAYS,
    configured_version_for,
)
from infra.ec2 import Ec2Port
from infra.emitter import EventEmitter
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.policies import ECR_TOKEN_POLICY, attach_policy
from infra.repository import Repository, registry_credentials
from infra.service_queue import ServiceConfiguration, ServiceQueue

import pulumi


class GraplDockerBuild(docker.DockerBuild):
    def __init__(
        self,
        dockerfile: str,
        target: str,
        context: Optional[str] = None,
        args: Optional[Mapping[str, pulumi.Input[str]]] = None,
        env: Optional[Mapping[str, str]] = None,
    ):
        super().__init__(
            context=context,
            dockerfile=dockerfile,
            env={**(env or {}), "DOCKER_BUILDKIT": 1},
            args={**(args or {}), "RUST_BUILD": "debug"},
            target=target,
            # Quiet the Docker builds at `pulumi up` time
            # ...except it doesn't work with `buildx` yet
            # https://github.com/docker/buildx/issues/401
            # extra_options=("--quiet",),
        )


class FargateTaskRole(aws.iam.Role):
    def __init__(
        self, name: str, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:
        super().__init__(
            f"{name}-task-role",
            description=f"Fargate task role for {name}",
            assume_role_policy=json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Action": "sts:AssumeRole",
                            "Principal": {"Service": "ecs-tasks.amazonaws.com"},
                        }
                    ],
                }
            ),
            opts=opts,
        )


class FargateExecutionRole(aws.iam.Role):
    def __init__(
        self, name: str, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:
        super().__init__(
            f"{name}-execution-role",
            description=f"Fargate execution role for {name}",
            assume_role_policy=json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Action": "sts:AssumeRole",
                            "Principal": {"Service": "ecs-tasks.amazonaws.com"},
                        }
                    ],
                }
            ),
            opts=opts,
        )


class _AWSFargateService(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        cluster: aws.ecs.Cluster,
        service_configuration: ServiceConfiguration,
        queue: ServiceQueue,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        network: Network,
        image: pulumi.Output[str],
        env: Mapping[str, Union[str, pulumi.Output[str]]],
        forwarder: MetricForwarder,
        entrypoint: Optional[List[str]] = None,
        command: Optional[List[str]] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        """
        :param command: supply an override to the CMD defined in the Dockerfile.
        """

        super().__init__("grapl:AWSFargateService", name, None, opts)

        self.task_role = FargateTaskRole(name, opts=pulumi.ResourceOptions(parent=self))

        ########################################################################
        # TODO: CDK code has us consuming from all queues, but that's
        # likely excessive. The default service probably just needs to
        # consume from the main queue; similarly for the retry service
        # and retry queue
        #
        # We should probably bundle this concept up into a single
        # policy (one for the "default" case and one for the "retry"
        # case), and then put this into the ServiceQueue object. Then,
        # anything that needs to behave as a "default service" can
        # just attach the appropriate policy; similarly for things
        # that behave like "retry services".
        #
        # That would likely allow us to unify the Fargate- and
        # Lambda-based services, too.
        queue.grant_main_queue_consumption_to(self.task_role)
        queue.grant_retry_queue_consumption_to(self.task_role)
        queue.grant_dead_letter_queue_consumption_to(self.task_role)
        ########################################################################

        ########################################################################
        # TODO: As above, we don't need everything to be able to send
        # to all our queues.
        #
        # If we take the approach advocated above with a single policy
        # laying out the behavior we want, then these attachments can
        # go away, since they will have been subsumed into the ones
        # above.
        queue.grant_main_queue_send_to(self.task_role)
        queue.grant_retry_queue_send_to(self.task_role)
        queue.grant_dead_letter_queue_send_to(self.task_role)
        ########################################################################

        input_emitter.grant_read_to(self.task_role)
        output_emitter.grant_write_to(self.task_role)

        self.execution_role = FargateExecutionRole(
            name, opts=pulumi.ResourceOptions(parent=self)
        )

        # Incorporating the stack name into this log group name;
        # otherwise we'll end up dumping logs from different stacks
        # together.
        #
        # TODO: Consider a helper function for generating log group
        # names that adheres to this convention for all our services
        # (though this will be less of an issue once we migrate to
        # Kafka)
        self.log_group = aws.cloudwatch.LogGroup(
            f"{name}-log-group",
            name=f"/grapl/{DEPLOYMENT_NAME}/{name}",
            retention_in_days=SERVICE_LOG_RETENTION_DAYS,
            opts=pulumi.ResourceOptions(parent=self),
        )

        aws.iam.RolePolicy(
            f"{name}-write-log-events",
            role=self.execution_role.name,
            policy=self.log_group.arn.apply(
                lambda arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Effect": "Allow",
                                "Action": ["logs:CreateLogStream", "logs:PutLogEvents"],
                                "Resource": f"{arn}:*",
                            }
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=self.execution_role),
        )

        # This is only needed if we're actually pulling from ECR,
        # which we don't do in production (because we're pulling from
        # Cloudsmith). The only time we use ECR is when we build a
        # Docker container locally, and that'll only happen for
        # individual developer sandbox deployments.
        # TODO: This feels hacky; consider other ways to model this.
        if not REAL_DEPLOYMENT:
            attach_policy(ECR_TOKEN_POLICY, self.execution_role)

        forwarder.subscribe_to_log_group(name, self.log_group)

        self.task = aws.ecs.TaskDefinition(  # type: ignore[call-overload]
            f"{name}-task",
            family=f"{DEPLOYMENT_NAME}-{name}-task",
            container_definitions=pulumi.Output.all(
                queue_url=service_configuration.main_url,
                dead_letter_url=service_configuration.dead_letter_url,
                log_group=self.log_group.name,
                bucket=output_emitter.bucket.bucket,
                image=image,
                env=env,
            ).apply(
                lambda inputs: json.dumps(
                    [
                        {
                            # NOTE: it seems that *all* our containers
                            # are named this. Perhaps due to CDK's
                            # QueueProcessingFargateService abstraction?
                            "name": "QueueProcessingContainer",
                            "image": inputs["image"],
                            "environment": _environment_from_map(
                                {
                                    "QUEUE_URL": inputs["queue_url"],
                                    "SOURCE_QUEUE_URL": inputs["queue_url"],
                                    "DEST_BUCKET_NAME": inputs["bucket"],
                                    "DEPLOYMENT_NAME": DEPLOYMENT_NAME,
                                    "DEAD_LETTER_QUEUE_URL": inputs["dead_letter_url"],
                                    **inputs["env"],
                                }
                            ),
                            "logConfiguration": {
                                "logDriver": "awslogs",
                                "options": {
                                    "awslogs-stream-prefix": "logs",
                                    "awslogs-region": aws.get_region().name,
                                    "awslogs-group": inputs["log_group"],
                                },
                            },
                            **({"entryPoint": entrypoint} if entrypoint else {}),
                            **({"command": command} if command else {}),
                        },
                    ]
                )
            ),
            requires_compatibilities=["FARGATE"],
            cpu=256,
            memory=512,
            network_mode="awsvpc",  # only option for Fargate
            task_role_arn=self.task_role.arn,
            execution_role_arn=self.execution_role.arn,
            opts=pulumi.ResourceOptions(
                parent=self,
            ),
        )

        self.security_group = aws.ec2.SecurityGroup(
            f"{name}-security-group",
            vpc_id=network.vpc.id,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.service = aws.ecs.Service(
            f"{name}-service",
            cluster=cluster.arn,
            network_configuration=aws.ecs.ServiceNetworkConfigurationArgs(
                assign_public_ip=False,
                subnets=[net.id for net in network.private_subnets],
                security_groups=[self.security_group.id],
            ),
            launch_type="FARGATE",
            desired_count=1,  # TODO: Set this to 1 or 0 depending on default vs. retry
            deployment_minimum_healthy_percent=50,
            task_definition=self.task.arn,
            opts=pulumi.ResourceOptions(
                parent=self,
            ),
        )

        self.register_outputs({})


class FargateService(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        input_emitter: EventEmitter,
        output_emitter: EventEmitter,
        network: Network,
        image: docker.DockerBuild,
        env: Mapping[str, Union[str, pulumi.Output[str]]],
        forwarder: MetricForwarder,
        entrypoint: Optional[List[str]] = None,
        command: Optional[List[str]] = None,
        retry_image: Optional[docker.DockerBuild] = None,
        retry_entrypoint: Optional[List[str]] = None,
        retry_command: Optional[List[str]] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:FargateService", name, None, opts)

        self.queue = ServiceQueue(name, opts=pulumi.ResourceOptions(parent=self))
        self.queue.subscribe_to_emitter(input_emitter)

        self.ecs_cluster = aws.ecs.Cluster(
            f"{name}-cluster",
            opts=pulumi.ResourceOptions(parent=self),
        )

        # We're not calling this image, e.g., "foo-default" to account
        # for the (common) case that the corresponding retry service
        # uses the same image.
        (repository, image_name) = self._repository_and_image(name, image)

        self.default_service = _AWSFargateService(
            f"{name}-default",
            cluster=self.ecs_cluster,
            queue=self.queue,
            service_configuration=self.queue.default_service_configuration(),
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            network=network,
            image=image_name,
            entrypoint=entrypoint,
            command=command,
            env=env,
            forwarder=forwarder,
            opts=pulumi.ResourceOptions(parent=self),
        )
        if repository:
            repository.grant_access_to(self.default_service.execution_role)

        # If a separate retry image was provided, create a separate
        # repository for it; otherwise, reuse the existing repository
        # and image.
        retry_name = f"{name}-retry"
        (retry_repository, retry_image_name) = (
            self._repository_and_image(retry_name, retry_image)
            if retry_image
            else (repository, image_name)
        )

        self.retry_service = _AWSFargateService(
            retry_name,
            cluster=self.ecs_cluster,
            queue=self.queue,
            service_configuration=self.queue.retry_service_configuration(),
            input_emitter=input_emitter,
            output_emitter=output_emitter,
            network=network,
            image=retry_image_name,
            entrypoint=retry_entrypoint or entrypoint,
            command=retry_command or command,
            env=env,
            forwarder=forwarder,
            opts=pulumi.ResourceOptions(parent=self),
        )
        if retry_repository:
            retry_repository.grant_access_to(self.retry_service.execution_role)

        self.services = (self.default_service, self.retry_service)

        self._setup_default_ports()

        self.register_outputs({})

    def _setup_default_ports(self) -> None:
        """
        Can be overridden by subclasses. Most services are fine having an outbound 443.
        Has a cognate in service.py.
        """
        for svc in self.services:
            Ec2Port("tcp", 443).allow_outbound_any_ip(svc.security_group)

    def allow_egress_to_cache(self, cache: Cache) -> None:
        """
        Allow both the default and retry services to connect to the `cache`.
        """
        for svc in self.services:
            cache.allow_egress_to_cache_for(svc._name, svc.security_group)

    def _repository_and_image(
        self, name: str, build: docker.DockerBuild
    ) -> Tuple[Optional[Repository], pulumi.Output[str]]:

        version = configured_version_for(name)
        if version:
            image_name = f"docker.cloudsmith.io/grapl/raw/{name}:{version}"
            pulumi.info(f"Version found for {name}: {version} ({image_name})")
            # It's a bit of a bummer to need this cast :/
            return (None, cast(pulumi.Output[str], image_name))
        else:
            # create ECR, build image, push to ECR, return output
            pulumi.info(
                f"Version NOT found for {name}; performing local container image build"
            )

            repository = Repository(name, opts=pulumi.ResourceOptions(parent=self))

            image = docker.Image(
                name,
                image_name=repository.registry_qualified_name,
                build=build,
                registry=registry_credentials(),
                opts=pulumi.ResourceOptions(parent=self),
            )

            # The built image name will have a checksum appended to it,
            # thus eliminating the need to use tags.
            return (repository, image.image_name)


def _environment_from_map(env: Mapping[str, str]) -> Sequence[Mapping[str, str]]:
    """
    Generate a list of environment variable dictionaries for an ECS task container definition from a standard dictionary.
    """
    return [{"name": k, "value": v} for (k, v) in env.items()]
