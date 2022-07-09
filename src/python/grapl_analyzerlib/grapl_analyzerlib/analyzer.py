from __future__ import annotations

import atexit
import os
from functools import cache

import abc
from datetime import datetime, timedelta
from dataclasses import dataclass, field, InitVar
from typing import (
    Sequence,
    Optional,
    Type,
    cast,
    Any,
    Union,
    List,
    Literal,
    Final,
    Dict,
    final,
    Protocol,
    ContextManager,
)

from python_proto.graplinc.grapl.api.graph_query.v1beta1.messages import (
    GraphQueryClient,
    NodeQuery,
    NodeView,
    GraphView,
    Uid,
)
from python_proto.graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.messages import (
    LensRef,
    ExecutionHit,
    AnalyzerName,
)
from python_proto.graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.server import (
    AnalyzerService,
    AnalyzerServiceWrapper,
)


@final
@dataclass(slots=True)
class AnalyzerContext:
    _analyzer_name: AnalyzerName
    _graph_client: GraphQueryClient
    _start_time: datetime
    _allowed: dict[Uid, timedelta | None]

    def get_graph_client(self) -> GraphQueryClient:
        return self._graph_client

    def get_remaining_time(self) -> timedelta:
        now = datetime.now()
        if self._start_time + timedelta(seconds=30) > now:
            return timedelta()
        return datetime.now() - self._start_time

    def _reset_start_time(self) -> None:
        self._start_time = datetime.now()


class Analyzer(Protocol):
    @staticmethod
    def query() -> NodeQuery:
        """
        * Queries returned by this function will only be executed when one of the properties or edges it references is
            updated in the graph
        * Must be 'pure' - Grapl may only execute this once per Analyzer deployment
        * The Query must represent a single, connected graph
        """
        ...

    async def analyze(
        self, matched: NodeView, ctx: AnalyzerContext
    ) -> ExecutionHit | None:
        """
        Called every time the Queryable matched by `query_graph` matches
        Used for any subsequent analysis
        ```python3
        children = matched.get_children(ProcessQuery().with_process_name(eq="cmd.exe"))
        if children:
            return ExecutionHit(matched)
        ```
        """
        ...

    async def add_context(self, matched: NodeView, ctx: AnalyzerContext) -> None:
        """
        Called when `analyze` returns an `AnalyzerHit`.
        `matched` is the graph stored in the AnalyzerHit
        ```python3
        matched.get_children()
        ```
        """
        ...


def serve_analyzer(
    analyzer_name: AnalyzerName,
    analyzer: Analyzer,
) -> None:
    "Runs the gRPC machinery to orchestrate the Analyzer"

    graph_query_client = cast(GraphQueryClient, None)

    await AnalyzerService(
        _analyzer_name=analyzer_name,
        _analyzer=analyzer,
        _graph_query_client=graph_query_client,
    ).serve()

    return None
