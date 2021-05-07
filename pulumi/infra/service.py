import dataclasses
from typing import Mapping, Optional, Union

from infra.lambda_ import LambdaExecutionRole, PythonLambdaArgs
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.queue_driven_lambda import QueueDrivenLambda
from infra.service_queue import ServiceQueue

import pulumi


# TODO: Needs a VPC
class Service(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        forwarder: MetricForwarder,
        lambda_description: str,
        lambda_handler_fn: str,
        lambda_code_path: str,
        network: Network,
        env: Mapping[str, Union[str, pulumi.Output[str]]],
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

        self.handler = QueueDrivenLambda(
            f"{name}-Handler",
            self.queue.queue,
            args=args,
            forwarder=forwarder,
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        # Note: The retry handler gets twice the memory and twice the
        # timeout as the "normal" handler.
        self.retry_handler = QueueDrivenLambda(
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

        self.register_outputs({})
