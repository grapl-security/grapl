from __future__ import annotations

import json
import os

from concurrent import futures
from typing import (
    Any,
    Mapping,
    Iterator,
)

import grpc

from graplinc.grapl.api.suspicious_svchost_analyzer.suspicious_svchost_analyzer.v1beta1.suspicious_svchost_analyzer_pb2_grpc import (
    SuspiciousSvchostAnalyzerServicer,
    add_SuspiciousSvchostAnalyzerServicer_to_server,
)
from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.execution import ExecutionHit as AnalyzerExecutionHit,
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.prelude import AssetQuery, Not, ProcessQuery, ProcessView
from grapl_analyzerlib.subgraph_view import SubgraphView
from grapl_common.grapl_logger import get_module_grapl_logger
from python_proto.api import AnalyzeRequest, AnalyzeResponse, MergedNode, MergedEdge, MergedEdgeList, ExecutionHit, Lens

# Set up logger (this is for the whole file, including static methods)
LOGGER = get_module_grapl_logger()

class SuspiciousSvchost(Analyzer):
    def get_queries(self) -> OneOrMany[ProcessQuery]:
        invalid_parents = [
            Not("services.exe"),
            Not("smss.exe"),
            Not("ngentask.exe"),
            Not("userinit.exe"),
            Not("GoogleUpdate.exe"),
            Not("conhost.exe"),
            Not("MpCmdRun.exe"),
        ]

        return (
            ProcessQuery()
            .with_process_name(eq=invalid_parents)
            .with_children(ProcessQuery().with_process_name(eq="svchost.exe"))
            .with_asset(AssetQuery().with_hostname())
        )

    def on_response(self, response: ProcessView, output: Any):
        asset_id = response.get_asset().get_hostname()

        output.send(
            AnalyzerExecutionHit(
                analyzer_name="Suspicious svchost",
                node_view=response,
                risk_score=75,
                lenses=[("hostname", asset_id), ("analyzer_name", "SuspiciousSvchost")],
                risky_node_keys=[
                    # the asset and the process
                    response.get_asset().node_key,
                    response.node_key,
                ],
            )
        )


def accumulator():
    retval = []
    while 1:
        result = yield
        if result is None:
            break
        else:
            retval.append(result)
    yield retval


def parse_nodes(nodes: str) -> Mapping[str, MergedNode]:
    LOGGER.debug(f"nodes: {nodes}")
    parsed = json.loads(nodes)
    return {}


def parse_edges(edges: str) -> Mapping[str, MergedEdgeList]:
    LOGGER.debug(f"edges: {edges}")
    parsed = json.loads(edges)
    return {}


def to_execution_hit(analyzer_execution_hit: AnalyzerExecutionHit) -> ExecutionHit:
    return ExecutionHit(
        nodes=parse_nodes(analyzer_execution_hit.nodes),
        edges=parse_edges(analyzer_execution_hit.edges),
        analyzer_name=analyzer_execution_hit.analyzer_name,
        risk_score=analyzer_execution_hit.risk_score,
        lenses=[Lens(lens_type=l[0], lens_name=l[1]) for l in analyzer_execution_hit.lenses],
        risky_node_keys=analyzer_execution_hit.risky_node_keys or [],
    )


def handle(
    analyzer: SuspiciousSvchost,
    graph_client: GraphClient,
    request: AnalyzeRequest,
) -> Iterator[ExecutionHit]:
    LOGGER.debug(f"handling event {str(envelope)}")

    subgraph = SubgraphView.from_proto(graph_client, request.merged_graph)
    queries = analyzer.get_queries()

    for node in subgraph.node_iter():
        for query in queries:
            response = query.query_first(
                graph_client, contains_node_key=node.node_key
            )
            if response:
                LOGGER.info(f"suspicious svchost analyzer hit")
                acc = accumulator()
                next(acc)  # prime the coroutine
                analyzer.on_response(response, acc)
                yield from (to_execution_hit(r) for r in acc.send(None))


class Servicer(SuspiciousSvchostAnalyzerServicer):
    def __init__(self, graph_client: GraphClient, analyzer: SuspiciousSvchost) -> None:
        self.graph_client = graph_client
        self.analyzer = analyzer

    def analyze(self, request, context) -> AnalyzeResponse.proto_cls:
        return AnalyzeResponse(execution_hits=[
            execution_hit for execution_hit in handle(
                self.analyzer,
                self.graph_client,
                AnalyzeRequest.from_proto(request)
            )
        ]).into_proto()


def main() -> None:
    graph_client = GraphClient()
    analyzer = SuspiciousSvchost(graph_client=graph_client)
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    add_SuspiciousSvchostAnalyzerServicer_to_server(
        Servicer(graph_client, analyzer), server
    )
    server.add_insecure_port(
        os.environ["SUSPICIOUS_SVCHOST_ANALYZER_BIND_ADDRESS"]
    )
    server.start()
    server.wait_for_termination()


if __name__ == "__main__":
    main()
