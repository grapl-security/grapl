# mypy: ignore-errors
"""
^ Type checking doesn't exist yet for gRPC asyncio runtime, so
we are ignoring *the entire file*. 
https://github.com/nipunn1313/mypy-protobuf/pull/217
"""

import logging
from concurrent import futures
from dataclasses import dataclass
from typing import Any, Awaitable, Protocol

import grpc
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1 import analyzers_pb2 as proto
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2_grpc import (
    AnalyzerServiceServicer,
    add_AnalyzerServiceServicer_to_server,
)
from python_proto.api.plugin_sdk.analyzers.v1beta1 import messages as native

_cleanup_coroutines: list[Awaitable[Any]] = []


class AnalyzerService(Protocol):
    async def run_analyzer(
        self, request: native.RunAnalyzerRequest
    ) -> native.RunAnalyzerResponse:
        pass


@dataclass(frozen=True)
class AnalyzerServiceWrapper(AnalyzerServiceServicer):
    bind_address: str
    analyzer_service_impl: AnalyzerService

    async def RunAnalyzer(
        self,
        proto_request: proto.RunAnalyzerRequest,
        context: grpc.aio.ServicerContext,
    ) -> proto.RunAnalyzerResponse:
        native_request = native.RunAnalyzerRequest.from_proto(proto_request)
        native_response = await self.analyzer_service_impl.run_analyzer(native_request)
        return native_response.into_proto()

    async def serve(self) -> None:
        server = grpc.aio.server(futures.ThreadPoolExecutor(max_workers=10))
        add_AnalyzerServiceServicer_to_server(
            AnalyzerServiceWrapper(
                analyzer_service_impl=self.analyzer_service_impl,
            ),
            server,
        )
        server.add_insecure_port(self.bind_address)
        await server.start()

        async def server_graceful_shutdown() -> None:
            logging.info("Starting graceful shutdown...")
            await server.stop(5)

        _cleanup_coroutines.append(server_graceful_shutdown())
        await server.wait_for_termination()

        return None
