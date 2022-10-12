from __future__ import annotations

import asyncio
import os
from dataclasses import dataclass
from typing import Protocol

from grapl_plugin_sdk.analyzer.analyzer_context import AnalyzerContext
from grapl_plugin_sdk.analyzer.query_and_views import NodeQuery, NodeView
from grapl_plugin_sdk.analyzer.service_impl import AnalyzerServiceImpl
from python_proto.api.graph_query.v1beta1.client import GraphQueryClient
from python_proto.api.plugin_sdk.analyzers.v1beta1.messages import (
    AnalyzerName,
    ExecutionHit,
)
from python_proto.api.plugin_sdk.analyzers.v1beta1.server import (
    AnalyzerService,
    AnalyzerServiceWrapper,
)
from python_proto.client import GrpcClientConfig


# TODO: would be nice to have query() -> ProcessQuery, analyze(ProcessView, ...)
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
        Don't confuse "context" here with the AnalyzerContext argument;
        we use this function to add additional nodes and edges to the Lens
        to provide a fuller picture (aka, context) of the ExecutionHit.

        Called when `analyze` returns an `AnalyzerHit`.
        `matched` is the graph stored in the AnalyzerHit
        ```python3
        matched.get_children()
        ```
        """
        ...


@dataclass(frozen=True, slots=True)
class AnalyzerServiceConfig:
    bind_address: str

    @classmethod
    def from_env(cls) -> AnalyzerServiceConfig:
        return cls(bind_address=os.environ["PLUGIN_BIND_ADDRESS"])


def graph_query_proxy_address() -> str:
    # Ideally we'd specify GRAPH_QUERY_CLIENT_ADDRESS in env{}, but
    # due to HCL limitations + the fact each `graph-query-proxy` service
    # has a different name, we instead construct the CLIENT_ADDRESS at
    # runtime here instead.
    # https://github.com/hashicorp/nomad/issues/14813
    plugin_id = os.environ["PLUGIN_ID"]
    upstream_addr_env_var = f"NOMAD_UPSTREAM_ADDR_graph-query-proxy-{plugin_id}"
    upstream_addr = os.environ[upstream_addr_env_var]
    return f"http://{upstream_addr}"


def serve_analyzer(
    analyzer_name: AnalyzerName,
    analyzer: Analyzer,
    service_config: AnalyzerServiceConfig,
) -> None:
    """
    Runs the gRPC machinery to orchestrate the Analyzer
    """
    graph_query_client = GraphQueryClient.connect(
        client_config=GrpcClientConfig(
            address=graph_query_proxy_address(),
        )
    )

    impl: AnalyzerService = AnalyzerServiceImpl(
        _analyzer_name=analyzer_name,
        _analyzer=analyzer,
        _graph_query_client=graph_query_client,
    )

    servicer = AnalyzerServiceWrapper(
        bind_address=service_config.bind_address,
        analyzer_service_impl=impl,
    )

    loop = asyncio.get_event_loop()
    try:
        loop.run_until_complete(servicer.serve())
    finally:
        loop.run_until_complete(*servicer._cleanup_coroutines)
        loop.close()
