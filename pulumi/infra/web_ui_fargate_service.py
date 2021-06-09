import json
from typing import Mapping, Optional, Sequence, Tuple, Union

import pulumi_aws as aws
import pulumi_docker as docker
from infra.cache import Cache
from infra.config import DEPLOYMENT_NAME, SERVICE_LOG_RETENTION_DAYS
from infra.emitter import EventEmitter
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.policies import ECR_TOKEN_POLICY, attach_policy
from infra.repository import Repository, registry_credentials
from infra.service_queue import ServiceQueue
from typing_extensions import Literal

import pulumi

KnownDockerfile = Literal[
    "../src/rust/Dockerfile",
    "../src/python/Dockerfile",
]


class GraplDockerBuild(docker.DockerBuild):
    def __init__(
            self,
            dockerfile: KnownDockerfile,
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


class _WebUiAWSFargateService(pulumi.ComponentResource):
    def __init__(
            self,
            name: str,
            cluster: aws.ecs.Cluster,
            network: Network,
            image: pulumi.Output[str],
            command: str,
            env: Mapping[str, Union[str, pulumi.Output[str]]],
            forwarder: MetricForwarder,
            opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        super().__init__("grapl:WebUiAWSFargateService", name, None, opts)

        self.task_role = FargateTaskRole(name, opts=pulumi.ResourceOptions(parent=self))

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
        attach_policy(ECR_TOKEN_POLICY, self.execution_role)

        forwarder.subscribe_to_log_group(name, self.log_group)

        self.task = aws.ecs.TaskDefinition(  # type: ignore[call-overload]
            f"{name}-task",
            family=f"{DEPLOYMENT_NAME}-{name}-task",
            container_definitions=pulumi.Output.all(
                log_group=self.log_group.name,
                image=image,
                env=env,
            ).apply(
                lambda inputs: json.dumps(
                    [
                        {
                            # NOTE: it seems that *all* our containers
                            # are named this. Perhaps due to CDK's
                            # QueueProcessingFargateService abstraction?
                            "name": "WebUiContainer",
                            "command": [command],
                            "image": inputs["image"],
                            "environment": _environment_from_map(
                                {
                                    "DEPLOYMENT_NAME": DEPLOYMENT_NAME,
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


class WebUiFargateService(pulumi.ComponentResource):
    def __init__(
            self,
            name: str,
            network: Network,
            image: docker.DockerBuild,
            command: str,
            env: Mapping[str, Union[str, pulumi.Output[str]]],
            forwarder: MetricForwarder,
            opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:WebUiFargateService", name, None, opts)

        self.ecs_cluster = aws.ecs.Cluster(
            f"{name}-cluster",
            opts=pulumi.ResourceOptions(parent=self),
        )

        # We're not calling this image, e.g., "foo-default" to account
        # for the (common) case that the corresponding retry service
        # uses the same image.
        (repository, image_name) = self._repository_and_image(name, image)

        self.default_service = _WebUiAWSFargateService(
            f"{name}-default",
            cluster=self.ecs_cluster,
            network=network,
            image=image_name,
            command=command,
            env=env,
            forwarder=forwarder,
            opts=pulumi.ResourceOptions(parent=self),
        )
        repository.grant_access_to(self.default_service.execution_role)

        self.register_outputs({})

    def allow_egress_to_cache(self, cache: Cache) -> None:
        """
        Allow both the default and retry services to connect to the `cache`.
        """
        cache.allow_egress_to_cache_for(self.default_service._name, self.default_service.security_group)

    def _repository_and_image(
            self, name: str, build: docker.DockerBuild
    ) -> Tuple[Repository, pulumi.Output[str]]:

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
