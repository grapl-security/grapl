from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import final

from python_proto.api.graph_query_proxy.v1beta1.client import GraphQueryProxyClient
from python_proto.api.plugin_sdk.analyzers.v1beta1.messages import AnalyzerName
from python_proto.grapl.common.v1beta1.messages import Uid


@final
@dataclass(slots=True)
class AnalyzerContext:
    _analyzer_name: AnalyzerName
    _graph_client: GraphQueryProxyClient
    _start_time: datetime
    _allowed: dict[Uid, timedelta | None]

    def get_graph_client(self) -> GraphQueryProxyClient:
        return self._graph_client

    def get_remaining_time(self) -> timedelta:
        now = datetime.now()
        if self._start_time + timedelta(seconds=30) > now:
            return timedelta()
        return datetime.now() - self._start_time

    def _reset_start_time(self) -> None:
        self._start_time = datetime.now()
