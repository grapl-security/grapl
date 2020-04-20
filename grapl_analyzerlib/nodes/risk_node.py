from typing import *

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.nodes.comparators import Cmp, IntCmp, _int_cmps, StrCmp, _str_cmps, PropertyFilter
from grapl_analyzerlib.nodes.queryable import NQ, Queryable
from grapl_analyzerlib.nodes.viewable import Viewable

from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.nodes.viewable import EdgeViewT, ForwardEdgeView

IRiskView = TypeVar('IRiskView', bound='RiskView')
IRiskQuery = TypeVar('IRiskQuery', bound='RiskQuery')


class RiskQuery(Queryable['RiskView']):
    def __init__(self) -> None:
        super(RiskQuery, self).__init__(RiskView)
        self._risk_score = []  # type: List[List[Cmp[int]]]
        self._analyzer_name = []  # type: List[List[Cmp[str]]]

        self._risky_nodes = None  # type: Optional[NodeQuery]

    def with_analyzer_name(
            self: "NQ",
            eq: Optional[StrCmp] = None,
            contains: Optional[StrCmp] = None,
            ends_with: Optional[StrCmp] = None,
            starts_with: Optional[StrCmp] = None,
            regexp: Optional[StrCmp] = None,
            distance: Optional[Tuple[StrCmp, int]] = None,
    ) -> 'NQ':
        cast("RiskQuery", self)._analyzer_name.extend(
            _str_cmps(
                "analyzer_name",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_risk_score(
            self: 'NQ',
            eq: Optional['IntCmp'] = None,
            gt: Optional['IntCmp'] = None,
            lt: Optional['IntCmp'] = None,
    ) -> 'NQ':
        cast("RiskQuery", self)._risk_score.extend(_int_cmps("risk_score", eq, gt, lt))
        return self

    def with_risky_nodes(
            self: "NQ", 
            risky_nodes_query: Optional["NodeQuery"] = None
    ) -> "NQ":
        risky_nodes = risky_nodes_query or NodeQuery()  # type: NodeQuery

        risky_nodes._risks = cast("List[RiskQuery]", self)
        risky_nodes.set_forward_edge_filter(
            'risks', self
        )
        cast("List[RiskQuery]", self)._risky_nodes = risky_nodes
        return self

    def _get_unique_predicate(self) -> Optional[Tuple[str, "PropertyT"]]:
        return None

    def _get_node_type_name(self) -> str:
        return 'Risk'

    def _get_property_filters(self) -> Mapping[str, "PropertyFilter[Property]"]:
        props = {
            "node_key": self._node_key,
            "risk_score": self._risk_score,
            "analyzer_name": self._analyzer_name,
        }
        combined = {}
        for prop_name, prop_filter in props.items():
            if prop_filter:
                combined[prop_name] = cast("PropertyFilter[Property]", prop_filter)

        return combined

    def _get_forward_edges(self) -> Mapping[str, "Queryable"]:
        return dict()

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        reverse_edges = {"~risks": (self._risky_nodes, "risky_nodes")}

        return {
            fe[0]: (fe[1][0], fe[1][1])
            for fe in reverse_edges.items()
            if fe[1][0] is not None
        }


class RiskView(Viewable):

    def __init__(
            self,
            dgraph_client: GraphClient,
            node_key: str,
            uid: str,
            node_type: str,
            risk_score: Optional[int] = None,
            analyzer_name: Optional[str] = None,
            risky_nodes: Optional[List['NodeView']] = None
    ):
        super(RiskView, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid, node_type=node_type
        )
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.node_type = node_type
        self.risk_score = risk_score
        self.analyzer_name = analyzer_name

        self.risky_nodes = risky_nodes or []

    def get_risk_score(self) -> Optional[int]:
        if not self.risk_score:
            self.risk_score = cast(Optional[int], self.fetch_property("risk_score", int))
        return self.risk_score

    def get_analyzer_name(self) -> Optional[str]:
        if not self.analyzer_name:
            self.analyzer_name = cast(Optional[str], self.fetch_property("analyzer_name", str))
        return self.analyzer_name

    def get_risky_nodes(
            self: "NV",
            match_risky_nodes: Optional[Queryable] = None
    ) -> Optional[str]:
        cast(ProcessView, self).risky_nodes = cast(
            RiskView, self.fetch_edges("~risks", type(self))
        )
        return cast(RiskView, self).risky_nodes

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            'risk_score': int,
            'analyzer_name': str,
        }

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        f_edges = {
        }  # type: Dict[str, Optional["EdgeViewT"]]
        return cast(Mapping[str, "EdgeViewT"], {
            fe[0]: fe[1] for fe in f_edges.items() if fe[1]
        })

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {
        }  # type: Dict[str, Optional[ForwardEdgeView]]
        return cast(
            "Mapping[str, ForwardEdgeView]",
            {fe[0]: fe[1] for fe in f_edges.items() if fe[1]}
        )

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        props = {
            'risk_score': self.risk_score,
            'analyzer_name': self.analyzer_name,
        }
        return {p[0]: p[1] for p in props.items() if p[1] is not None}

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        return {"~risks": (NodeView, "risky_nodes")}

    def _get_reverse_edges(self) -> "Mapping[str,  ReverseEdgeView]":
        _reverse_edges = {"~risks": (NodeView, "risky_nodes")}

        reverse_edges = {
            name: value
            for name, value in _reverse_edges.items()
            if value[0] is not None
        }
        return cast("Mapping[str, ReverseEdgeView]", reverse_edges)

    def get_node_type(self) -> str:
        return 'Risk'


from grapl_analyzerlib.nodes.any_node import NodeView, NodeQuery
from grapl_analyzerlib.nodes.process_node import ProcessView