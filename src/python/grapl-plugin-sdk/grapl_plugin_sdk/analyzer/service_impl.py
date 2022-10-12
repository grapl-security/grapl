from __future__ import annotations

import dataclasses
import logging
import os
import sys
from dataclasses import dataclass, field
from datetime import datetime
from typing import TYPE_CHECKING, Final, Protocol, runtime_checkable

import grpc
from grapl_plugin_sdk.analyzer.analyzer_context import AnalyzerContext

if TYPE_CHECKING:
    from grapl_plugin_sdk.analyzer.analyzer import Analyzer

from uuid import UUID

from grapl_plugin_sdk.analyzer.query_and_views import NodeView
from grpc import aio as grpc_aio  # type: ignore

# ^ grpc_aio: Type checking doesn't exist yet for gRPC asyncio runtime
from python_proto.api.graph_query.v1beta1 import messages as graph_query_messages
from python_proto.api.graph_query_proxy.v1beta1.client import GraphQueryProxyClient
from python_proto.api.plugin_sdk.analyzers.v1beta1 import messages as analyzer_messages
from python_proto.common import Uuid as PythonProtoUuid
from python_proto.grapl.common.v1beta1 import messages as grapl_common_messages

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.environ["ANALYZER_LOG_LEVEL"])
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


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


@dataclass(slots=True)
class AnalyzerServiceImpl:
    _analyzer: Analyzer
    _analyzer_name: analyzer_messages.AnalyzerName
    _graph_client: GraphQueryProxyClient
    _graph_query: graph_query_messages.GraphQuery = field(init=False)

    def __post_init__(self) -> None:
        self._graph_query = self._analyzer.query().into_graph_query()

    async def run_analyzer(
        self,
        request: analyzer_messages.RunAnalyzerRequest,
        context: grpc_aio.ServicerContext,
    ) -> analyzer_messages.RunAnalyzerResponse:
        try:
            return await self._run_analyzer_inner(request)
        except Exception as e:
            details = f"error_as_grpc_abort exception: {str(e)}"
            code = grpc.StatusCode.UNKNOWN
            await context.abort(
                code=code,
                details=details,
            )
        raise AssertionError("not reachable")

    async def _run_analyzer_inner(
        self, request: analyzer_messages.RunAnalyzerRequest
    ) -> analyzer_messages.RunAnalyzerResponse:
        match request.update.inner:
            case PropertyUpdate() as prop_update:
                LOGGER.debug("PropertyUpdate")
                # optimization
                # i.e. if the update is for process_name, and you're not querying for
                # process_name, that's obviously a miss
                prop_name = prop_update.property_name
                if not check_for_string_property(self._graph_query, prop_name):
                    LOGGER.debug("No string property")
                    return MISS_RESPONSE

                updated_node_uid = prop_update.uid
            case analyzer_messages.EdgeUpdate() as edge_update:
                raise NotImplementedError(
                    "I'll implement Edge Updates after we can do tests"
                )
                # updated_node_uid = TODO

        # Now we have the UID of nodes recently updated, and a
        # query. check if the UID could match any in the query.
        matched_graph: graph_query_messages.MatchedGraphWithUid | None = (
            self._graph_client.query_with_uid(
                node_uid=updated_node_uid,
                graph_query=self._graph_query,
            ).maybe_match.as_optional()
        )
        # if matched_graphs empty, that's a textbook miss
        if not matched_graph:
            LOGGER.debug("No matching graph, returning ExecutionMiss")
            return MISS_RESPONSE

        graph_view: graph_query_messages.GraphView = matched_graph.matched_graph
        root_uid = matched_graph.root_uid

        root_node_properties = graph_view.get_node(root_uid)
        if not root_node_properties:
            # todo: log this, it's an error
            # todo: return an error
            LOGGER.debug("(This should be an error)")
            return MISS_RESPONSE

        analyzer = self._analyzer
        ctx = self._new_ctx()

        root_node = NodeView.from_parts(
            root_node_properties,
            graph_view,
            self._graph_client,
            tenant_id=TENANT_ID,
        )

        # todo: Add a timeout here
        execution_hit: analyzer_messages.ExecutionHit | None = await analyzer.analyze(
            root_node, ctx
        )
        if not execution_hit:
            LOGGER.debug("No execution hit after calling analyze()")
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
