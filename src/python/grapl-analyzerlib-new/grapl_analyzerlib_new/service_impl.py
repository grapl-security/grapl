from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from typing import Final, Protocol, runtime_checkable

from grapl_analyzerlib_new.analyzer import Analyzer, AnalyzerContext
from grapl_analyzerlib_new.query_and_views import NodeView
from python_proto.api.graph_query.v1beta1 import messages as graph_query_messages
from python_proto.api.graph_query.v1beta1.client import GraphQueryClient
from python_proto.api.plugin_sdk.analyzers.v1beta1 import messages as analyzer_messages
from python_proto.grapl.common.v1beta1 import messages as grapl_common_messages


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
    _graph_query_client: GraphQueryClient
    _graph_query: graph_query_messages.GraphQuery = field(init=False)

    def __post_init__(self) -> None:
        self._graph_query = self._analyzer.query().into_graph_query()

    async def run_analyzer(
        self, request: analyzer_messages.RunAnalyzerRequest
    ) -> analyzer_messages.RunAnalyzerResponse:
        tenant_id = request.tenant_id
        update = request.update

        if isinstance(update.inner, PropertyUpdate) and (
            prop_name := update.inner.property_name
        ):
            if not check_for_string_property(self._graph_query, prop_name):
                return MISS_RESPONSE

        root_uid = update.inner.uid
        matched_graph: graph_query_messages.MatchedGraphWithUid | None = (
            self._graph_query_client.query_with_uid(
                tenant_id=tenant_id,
                node_uid=root_uid,
                graph_query=self._graph_query,
            ).maybe_match.as_optional()
        )
        if not matched_graph:
            return MISS_RESPONSE

        graph_view: graph_query_messages.GraphView = matched_graph.matched_graph

        root_node_properties = graph_view.get_node(root_uid)
        if not root_node_properties:
            # todo: log this, it's an error
            # todo: return an error
            return MISS_RESPONSE

        analyzer = self._analyzer
        ctx = self._new_ctx()

        root_node = NodeView.from_parts(
            root_node_properties,
            graph_view,
            self._graph_query_client,
            tenant_id=tenant_id,
        )

        # todo: Add a timeout here
        execution_hit: analyzer_messages.ExecutionHit | None = await analyzer.analyze(
            root_node, ctx
        )
        if not execution_hit:
            return MISS_RESPONSE

        await analyzer.add_context(root_node, ctx)

        return analyzer_messages.RunAnalyzerResponse(
            execution_result=analyzer_messages.ExecutionResult(
                inner=execution_hit,
            )
        )

    def _new_ctx(self) -> AnalyzerContext:
        return AnalyzerContext(
            _analyzer_name=self._analyzer_name,
            _graph_client=self._graph_query_client,
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
