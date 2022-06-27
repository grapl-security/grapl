# from __future__ import annotations
#
# import grpc
# import abc
# import datetime
# import uuid
# from dataclasses import dataclass
# from typing import Sequence, Optional, Type, cast, Any, Union
#
# from python_proto.graplinc.grapl.api.graph_query.v1beta1.graph_query import GraphView, GraphQueryClient, NodeQuery, NodeView, Uid, \
#     NodeType, PropertyName, EdgeName
# from proto.graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2_grpc import AnalyzerServiceServicer
#
# from proto.graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.analyzers_pb2 import RunAnalyzerRequest, RunAnalyzerResponse
#
#
# class AnalyzerContext(object):
#     def get_graph_client(self) -> GraphQueryClient:
#         raise NotImplementedError
#
#     def get_remaining_time(self) -> datetime.timedelta:
#         raise NotImplementedError
#
#
# class AnalyzerHit(object):
#     def __init__(self, graph: GraphView, score: int, lenses: Sequence[LensRef, ...]) -> None:
#         """
#         `graph` - A graph that has been definitively marked as being suspicious
#         `score` - Non-Zero positive integer representing how suspicious the graph is
#         `lenses` - A Non-Empty Sequence of Lenses to attach these nodes to
#         """
#         ...
#
#
# class LensRef(object):
#     def __init__(self, lens_type: str, lens_name: str) -> None:
#         """
#         A reference to a lens that may or may not exist.
#         """
#         ...
#
#
# class Analyzer(object):
#     _analyzer_context: AnalyzerContext
#
#     def _set_analyzer_context(self, ctx: AnalyzerContext) -> None:
#         self._analyzer_context = ctx
#
#     def get_analyzer_context(self) -> AnalyzerContext:
#         return self._analyzer_context
#
#     @staticmethod
#     @abc.abstractmethod
#     def query() -> NodeQuery:
#         """
#         * Queries returned by this function will only be executed when one of the properties or edges it references is updated in the graph
#         * Must be 'pure' - Grapl may only execute this once per Analyzer deployment
#         * The Query must represent a single, connected graph
#         """
#         ...
#
#     def analyze(self, matched: NodeView) -> Optional[AnalyzerHit]:
#         """
#         Called every time the Queryable matched by `query_graph` matches
#         Used for any subsequent analysis
#         ```python3
#         children = matched.get_children(ProcessQuery().with_process_name(eq="cmd.exe"))
#         if children:
#             return ExecutionHit(matched)
#         ```
#         """
#         ...
#
#     def context(self, matched: NodeView) -> None:
#         """
#         Called when `analyze` returns an `AnalyzerHit`.
#         `matched` is the graph stored in the AnalyzerHit
#         ```python3
#         matched.get_children()
#         ```
#         """
#         ...
#
#     def mark_allowed(self, view: NodeView, allowed_for: Optional[datetime.timedelta]=None) -> None:
#         """
#         Given a node, marks that node such that it will never trigger this Analyzer again. Optionally accepts a duration for which to allow for.
#         ex: A VirusTotal Analyzer may want to avoid scanning a node more than 1x a day
#         """
#         ...
#
#
# class AnalyzerServiceImpl(AnalyzerServiceServicer):
#     def __init__(self, analyzer: Analyzer) -> None:
#         self.analyzer = analyzer
#
#     def RunAnalyzer(self,
#                     request: RunAnalyzerRequest,
#                     context: grpc.ServicerContext,
#                     ) -> RunAnalyzerResponse:
#         ...
#
# # def serve_analyzer(analyzer: Analyzer):
# #     "Runs the gRPC machinery to orchestrate the Analyzer"
# #     next_context: AnalyzerContext = cast(AnalyzerContext, ())
