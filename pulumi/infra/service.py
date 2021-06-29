import dataclasses
from typing import Mapping, Optional, Sequence, Union

from infra.ec2 import Ec2Port
from infra.lambda_ import LambdaExecutionRole, PythonLambdaArgs
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.queue_driven_lambda import QueueDrivenLambda
from infra.service_queue import ServiceQueue
from typing_extensions import Protocol

import pulumi


class ServiceLike(Protocol):
    """
    Describes shared properties between Service and FargateService.
    """

    queue: ServiceQueue


class Service(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        forwarder: MetricForwarder,
        lambda_handler_fn: str,
        lambda_code_path: str,
        network: Network,
        env: Mapping[str, Union[str, pulumi.Output[str]]],
        lambda_description: Optional[str] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:Service", name, None, opts)

        self.name = name

        self.queue = ServiceQueue(name, opts=pulumi.ResourceOptions(parent=self))

        # Handlers *and* retry handlers both use this same role
        self.role = LambdaExecutionRole(
            name,
            opts=pulumi.ResourceOptions(parent=self),
        )

        handler_memory = 128  # MB
        handler_timeout = 45  # Seconds

        args = PythonLambdaArgs(
            execution_role=self.role,
            description=lambda_description,
            handler=lambda_handler_fn,
            code_path=lambda_code_path,
            memory_size=handler_memory,
            timeout=handler_timeout,
            env={**env, "SOURCE_QUEUE_URL": self.queue.queue.id, "IS_RETRY": "False"},
        )

        self._handler = QueueDrivenLambda(
            f"{name}-Handler",
            self.queue.queue,
            args=args,
            forwarder=forwarder,
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        # Note: The retry handler gets twice the memory and twice the
        # timeout as the "normal" handler.
        self._retry_handler = QueueDrivenLambda(
            f"{name}-RetryHandler",
            self.queue.retry_queue,
            args=dataclasses.replace(
                args,
                memory_size=handler_memory * 2,
                timeout=handler_timeout * 2,
                env={
                    **env,
                    "SOURCE_QUEUE_URL": self.queue.retry_queue.id,
                    "IS_RETRY": "True",
                },
            ),
            forwarder=forwarder,
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self._setup_default_ports()

        self.register_outputs({})

    def _setup_default_ports(self) -> None:
        """
        Can be overridden by subclasses. Most services are fine having an outbound 443.
        Has a cognate in fargate_service.py.
        """
        for handler in self.handlers:
            Ec2Port("tcp", 443).allow_outbound_any_ip(handler.function.security_group)

    @property
    def handlers(self) -> Sequence[QueueDrivenLambda]:
        return (self._handler, self._retry_handler)
