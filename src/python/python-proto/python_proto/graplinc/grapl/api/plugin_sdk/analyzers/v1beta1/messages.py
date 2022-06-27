# from __future__ import annotations
#
# import abc
# import datetime
# import uuid
# from dataclasses import dataclass
# from typing import Sequence, Optional, cast, Union
#
# from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2 import (
#     GraphView,
#     NodeView,
#     NodeQuery,
# )
#
# from python_proto.graplinc.grapl.api.graph_query.v1beta1.graph_query import StringOperation, Uid, NodeType, \
#     PropertyName, EdgeName
#
#
# class AnalyzerHit(object):
#     def __init__(
#             self,
#             graph: GraphView,
#             score: int,
#             lenses: Sequence[LensRef],
#     ) -> None:
#         """
#         `graph` - A graph that has been definitively marked as being suspicious
#         `score` - Non-Zero positive integer representing how suspicious the graph is
#         `lenses` - A Non-Empty Sequence of Lenses to attach these nodes to
#         """
#         self.graph = graph
#         self.score = score
#         self.lenses = lenses
#
#
# class LensRef(object):
#     def __init__(self, lens_type: str, lens_name: str) -> None:
#         """
#         A reference to a lens that may or may not exist.
#         """
#         self.lens_type = lens_type
#         self.lens_name = lens_name
#
#
# @dataclass(frozen=True)
# class StrProperty:
#     value: str
#     conflict_resolution: StringOperation
#
#
# @dataclass(frozen=True)
# class IntProperty:
#     value: int
#
#
# @dataclass(frozen=True)
# class Property:
#     value: Union[StrProperty, IntProperty]
#
#
# @dataclass(frozen=True)
# class PropertyUpdate:
#     uid: Uid
#     node_type: NodeType
#     property_name: PropertyName
#     property_value: Property
#
#
# @dataclass(frozen=True)
# class EdgeUpdate:
#     src_uid: Uid
#     dst_uid: Uid
#     src_type: NodeType
#     dst_type: NodeType
#     fwd_edge_name: EdgeName
#     rvs_edge_name: EdgeName
#
#
# @dataclass(frozen=True)
# class Update:
#     update: Union[PropertyUpdate, EdgeUpdate]
#
#
# @dataclass(frozen=True)
# class RunAnalyzerRequest:
#     tenant_id: uuid.UUID
#     update: Update
#
#
# @dataclass(frozen=True)
# class RunAnalyzerResponse:
#     ...
#
