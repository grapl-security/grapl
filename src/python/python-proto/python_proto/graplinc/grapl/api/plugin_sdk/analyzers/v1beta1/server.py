import logging
from concurrent import futures
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Any, Awaitable, List

import graplinc
import grpc
from grapl_analyzerlib.analyzer import Analyzer, AnalyzerContext
from graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2_grpc import (
    AnalyzerServiceServicer,
    add_AnalyzerServiceServicer_to_server,
)
from python_proto.graplinc.grapl.api.graph_query.v1beta1.messages import (
    GraphQuery,
    GraphQueryClient,
    GraphView,
    NodeView,
    PropertyName,
)
from python_proto.graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.messages import (
    AnalyzerName,
    ExecutionHit,
    ExecutionResult,
    RunAnalyzerRequest,
    RunAnalyzerResponse,
)

_cleanup_coroutines: list[Awaitable[Any]] = []


@dataclass(slots=True)
class AnalyzerService:
    _analyzer_name: AnalyzerName
    _analyzer: Analyzer
    _graph_query: GraphQuery = field(init=False)
    _graph_query_client: GraphQueryClient

    def __post_init__(self) -> None:
        self._graph_query = GraphQuery.from_node_query(self._analyzer.query())

    async def run_analyzer(self, request: RunAnalyzerRequest) -> RunAnalyzerResponse:
        _tenant_id = request.tenant_id
        update = request.update

        if not check_for_string_property(self._graph_query, update.inner.property_name):
            return RunAnalyzerResponse.miss()

        root_uid = update.inner.uid
        graph_view: GraphView | None = self._graph_query_client.query_with_uid(
            node_uid=root_uid,
            graph_query=self._graph_query,
        )
        if not graph_view:
            return RunAnalyzerResponse.miss()

        root_node_properties = graph_view.get_node(root_uid)
        if not root_node_properties:
            # todo: log this, it's an error
            # todo: return an error
            return RunAnalyzerResponse.miss()

        analyzer = self._analyzer
        ctx = self._new_ctx()

        root_node = NodeView.from_parts(
            root_node_properties,
            graph_view,
            self._graph_query_client,
        )

        # todo: Add a timeout here
        execution_hit: ExecutionHit | None = await analyzer.analyze(root_node, ctx)
        if not execution_hit:
            return RunAnalyzerResponse.miss()

        await analyzer.add_context(root_node, ctx)

        return RunAnalyzerResponse(
            execution_result=ExecutionResult(
                inner=execution_hit,
            )
        )

    async def serve(self) -> None:
        server = grpc.aio.server(futures.ThreadPoolExecutor(max_workers=10))
        add_AnalyzerServiceServicer_to_server(
            AnalyzerServiceWrapper(
                analyzer_service_impl=self,
            ),
            server,
        )
        listen_addr = "[::]:50051"
        server.add_insecure_port(listen_addr)
        await server.start()

        async def server_graceful_shutdown() -> None:
            logging.info("Starting graceful shutdown...")
            await server.stop(5)

        _cleanup_coroutines.append(server_graceful_shutdown())
        await server.wait_for_termination()

        return None

    def _new_ctx(self) -> AnalyzerContext:
        return AnalyzerContext(
            _analyzer_name=self._analyzer_name,
            _graph_client=self._graph_query_client,
            _start_time=datetime.now(),
            _allowed={},
        )


@dataclass(frozen=True)
class AnalyzerServiceWrapper(AnalyzerServiceServicer):
    analyzer_service_impl: AnalyzerService

    async def RunAnalyzer(
        self,
        request: graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2.RunAnalyzerRequest,
        context: grpc.aio.ServicerContext,  # type: ignore
    ) -> graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2.RunAnalyzerResponse:
        response = await self.analyzer_service_impl.run_analyzer(
            RunAnalyzerRequest.from_proto(request)
        )
        return response.into_proto()


def check_ctx_timeout(ctx: AnalyzerContext) -> RunAnalyzerResponse | None:
    if ctx.get_remaining_time() == timedelta():
        return RunAnalyzerResponse.miss()
    else:
        return None


def check_for_string_property(
    graph_query: GraphQuery,
    property_name: PropertyName,
) -> bool:
    for node in graph_query.node_property_queries.entries.values():
        for query_property_name in node.string_filters.keys():
            if property_name == query_property_name:
                return True
    return False
