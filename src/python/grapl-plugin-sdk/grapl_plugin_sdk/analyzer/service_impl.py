from __future__ import annotations

import dataclasses
import os
from dataclasses import dataclass, field
from datetime import datetime
from typing import TYPE_CHECKING, Final, Protocol, runtime_checkable
from uuid import UUID

import grpc
from grapl_common.logger import Structlogger, get_structlogger
from grapl_plugin_sdk.analyzer.analyzer_context import AnalyzerContext
from grapl_plugin_sdk.analyzer.query_and_views import NodeView
from grpc import aio as grpc_aio  # type: ignore
from python_proto.api.graph_query.v1beta1 import messages as graph_query_messages
from python_proto.api.graph_query_proxy.v1beta1.client import GraphQueryProxyClient
from python_proto.api.plugin_sdk.analyzers.v1beta1 import messages as analyzer_messages
from python_proto.common import Uuid as PythonProtoUuid
from python_proto.grapl.common.v1beta1 import messages as grapl_common_messages

# ^ grpc_aio: Type checking doesn't exist yet for gRPC asyncio runtime
from python_proto.metadata import GrpcOutboundMetadata

if TYPE_CHECKING:
    from grapl_plugin_sdk.analyzer.analyzer import Analyzer


LOGGER = get_structlogger()


def _get_tenant_id() -> PythonProtoUuid:
    env_var: str = os.environ["TENANT_ID"]  # specified in hax_docker_analyzer.nomad
    py_native_uuid = UUID(env_var)
    return PythonProtoUuid.from_uuid(py_native_uuid)


TENANT_ID: Final[PythonProtoUuid] = _get_tenant_id()


@runtime_checkable
class PropertyUpdate(Protocol):
    """
    a protocol that captures
    StringPropertyUpdate,
    Int64PropertyUpdate,
    UInt64PropertyUpdate

    and, thanks to `@runtime_checkable`, lets us do isinstance checks!
    """

    property_name: grapl_common_messages.PropertyName
    uid: grapl_common_messages.Uid


MISS_RESPONSE: Final[
    analyzer_messages.RunAnalyzerResponse
] = analyzer_messages.RunAnalyzerResponse(
    execution_result=analyzer_messages.ExecutionResult(
        inner=analyzer_messages.ExecutionMiss()
    )
)


class RunAnalyzerRequestMetadata:
    trace_id: str | None

    def __init__(self, grpc_metadata: grpc_aio.Metadata | None) -> None:
        grpc_metadata = grpc_metadata or {}
        self.trace_id = grpc_metadata.get("x-trace-id")

    def to_grpc_metadata(self) -> GrpcOutboundMetadata:
        metadata: GrpcOutboundMetadata = {}
        if self.trace_id:
            metadata["x-trace-id"] = self.trace_id
        # add more as needed
        return metadata


@dataclass(slots=True)
class AnalyzerServiceImpl:
    _analyzer: Analyzer
    _analyzer_name: analyzer_messages.AnalyzerName
    _graph_client: GraphQueryProxyClient
    _graph_query: graph_query_messages.GraphQuery = field(init=False)

    def __post_init__(self) -> None:
        self._graph_query = self._analyzer.query().into_graph_query()
        LOGGER.info("Graph query", graph_query=str(self._graph_query))

    async def run_analyzer(
        self,
        request: analyzer_messages.RunAnalyzerRequest,
        context: grpc_aio.ServicerContext,
    ) -> analyzer_messages.RunAnalyzerResponse:

        # TODO: Extract a Request ID from context.invocation_metadata()
        metadata = RunAnalyzerRequestMetadata(
            grpc_aio.Metadata.from_tuple(context.invocation_metadata())
        )
        logger = LOGGER.bind(
            trace_id=str(metadata.trace_id),
        )
        logger.debug("run_analyzer on request", request=request)

        try:
            return await self._run_analyzer_inner(request, logger, metadata)
        except Exception as e:
            logger.error("run_analyzer failed", error=str(e))
            details = f"error_as_grpc_abort exception: {str(e)}"
            code = grpc.StatusCode.UNKNOWN
            await context.abort(
                code=code,
                details=details,
            )
        raise AssertionError("not reachable")

    async def _run_analyzer_inner(
        self,
        request: analyzer_messages.RunAnalyzerRequest,
        logger: Structlogger,
        metadata: RunAnalyzerRequestMetadata,
    ) -> analyzer_messages.RunAnalyzerResponse:
        match request.update.inner:
            case PropertyUpdate() as prop_update:
                # optimization
                # i.e. if the update is for process_name, and you're not querying for
                # process_name, that's obviously a miss
                prop_name = prop_update.property_name
                if not check_for_string_property(self._graph_query, prop_name):
                    logger.debug(
                        "This PropertyUpdate is not for a StringProperty we're querying on."
                    )
                    return MISS_RESPONSE

                updated_node_uid = prop_update.uid
            case analyzer_messages.EdgeUpdate() as edge_update:
                # future potential optimization: add an
                # if not query_pertains_to_edge_update(self._graph_query, edge_update):
                #   return MISS_RESPONSE
                # where the function checks the names of update's edges against the graph query's edges

                updated_node_uid = edge_update.src_uid

        # Now we have the UID of nodes recently updated, and a
        # query. check if the UID could match any in the query.

        matched_graph: graph_query_messages.MatchedGraphWithUid | None = (
            self._graph_client.query_with_uid(
                node_uid=updated_node_uid,
                graph_query=self._graph_query,
                metadata=metadata.to_grpc_metadata(),
            ).maybe_match.as_optional()
        )
        # if matched_graphs empty, that's a textbook miss
        if not matched_graph:
            logger.debug("No matching graph, returning ExecutionMiss")
            return MISS_RESPONSE

        graph_view: graph_query_messages.GraphView = matched_graph.matched_graph
        root_uid = matched_graph.root_uid

        root_node_properties = graph_view.get_node(root_uid)
        if not root_node_properties:
            # todo: log this, it's an error
            # todo: return an error
            logger.debug("(This should be an error)")
            return MISS_RESPONSE

        analyzer = self._analyzer
        ctx = self._new_ctx()

        root_node = NodeView.from_parts(
            root_node_properties,
            graph_view,
            self._graph_client,
            tenant_id=TENANT_ID,
        )

        execution_hit: analyzer_messages.ExecutionHit | None = await analyzer.analyze(
            root_node, ctx
        )
        if not execution_hit:
            logger.debug("No execution hit after calling analyze()")
            return MISS_RESPONSE
        execution_hit = dataclasses.replace(
            execution_hit, analyzer_name=self._analyzer_name
        )

        await analyzer.add_context(root_node, ctx)

        return analyzer_messages.RunAnalyzerResponse(
            execution_result=analyzer_messages.ExecutionResult(
                inner=execution_hit,
            )
        )

    def _new_ctx(self) -> AnalyzerContext:
        return AnalyzerContext(
            _analyzer_name=self._analyzer_name,
            _graph_client=self._graph_client,
            _start_time=datetime.now(),
            _allowed={},
        )


def check_for_string_property(
    graph_query: graph_query_messages.GraphQuery,
    property_name: grapl_common_messages.PropertyName,
) -> bool:
    for node in graph_query.node_property_queries.entries.values():
        for query_property_name in node.string_filters.keys():
            if property_name == query_property_name:
                return True
    return False
