# mypy: ignore-errors
"""
^ Type checking doesn't exist yet for gRPC asyncio runtime, so
we are ignoring *the entire file*. 
https://github.com/nipunn1313/mypy-protobuf/pull/217
"""

import logging
import os
import sys
from concurrent import futures
from dataclasses import dataclass, field
from typing import Any, Awaitable, Protocol

import grpc
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1 import analyzers_pb2 as proto
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2_grpc import (
    AnalyzerServiceServicer,
    add_AnalyzerServiceServicer_to_server,
)
from grpc_health.v1 import health, health_pb2_grpc
from python_proto.api.plugin_sdk.analyzers.v1beta1 import messages as native

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.environ["ANALYZER_LOG_LEVEL"])
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


class AnalyzerService(Protocol):
    async def run_analyzer(
        self, request: native.RunAnalyzerRequest, context: grpc.aio.ServicerContext
    ) -> native.RunAnalyzerResponse:
        pass


@dataclass(slots=True, frozen=True)
class AnalyzerServiceWrapper(AnalyzerServiceServicer):
    bind_address: str
    analyzer_service_impl: AnalyzerService
    _cleanup_coroutines: list[Awaitable[Any]] = field(default_factory=list)

    async def RunAnalyzer(
        self,
        proto_request: proto.RunAnalyzerRequest,
        context: grpc.aio.ServicerContext,
    ) -> proto.RunAnalyzerResponse:
        native_request = native.RunAnalyzerRequest.from_proto(proto_request)
        LOGGER.info("Running analyzer")
        LOGGER.debug(f"Analyzer request: {native_request}")
        native_response = await self.analyzer_service_impl.run_analyzer(
            native_request, context
        )
        LOGGER.info("Analyzer run complete")
        LOGGER.debug(f"Analyzer response: {native_response}")
        return native_response.into_proto()

    async def serve(self) -> None:
        server = grpc.aio.server(futures.ThreadPoolExecutor(max_workers=10))
        add_AnalyzerServiceServicer_to_server(
            self,
            server,
        )
        health_pb2_grpc.add_HealthServicer_to_server(health.HealthServicer(), server)
        server.add_insecure_port(self.bind_address)
        # This shutdown stuff is suggested in the grpc example here:
        # https://github.com/grpc/grpc/blob/master/examples/python/helloworld/async_greeter_server_with_graceful_shutdown.py
        async def server_graceful_shutdown() -> None:
            LOGGER.info("Starting graceful shutdown...")
            await server.stop(5)

        self._cleanup_coroutines.append(server_graceful_shutdown())

        # Uncomment to experiment with grpc reflection.
        from grpc_reflection.v1alpha import reflection
        SERVICE_NAMES = (
            proto.DESCRIPTOR.services_by_name['AnalyzerService'].full_name,
            reflection.SERVICE_NAME,
        )
        reflection.enable_server_reflection(SERVICE_NAMES, server)

        LOGGER.info("Starting analyzer server")
        await server.start()
        LOGGER.info("Analyzer started, waiting for requests.")
        await server.wait_for_termination()
