from typing import *

from grapl_analyzerlib.nodes.queryable import Queryable, NQ
from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.nodes.viewable import EdgeViewT, ForwardEdgeView, Viewable, ReverseEdgeView
from grapl_analyzerlib.nodes.comparators import Cmp, IntCmp, _int_cmps, StrCmp, _str_cmps, PropertyFilter

from pydgraph import DgraphClient

IAssetQuery = TypeVar('IAssetQuery', bound='AssetQuery')
IAssetView = TypeVar('IAssetView', bound='AssetView')


class AssetQuery(Queryable):
    def __init__(self):
        super(AssetQuery, self).__init__(AssetView)

        self._hostname = []  # type: List[List[Cmp[str]]]

    def _get_unique_predicate(self) -> Optional[Tuple[str, "PropertyT"]]:
        return None

    def _get_node_type_name(self) -> str:
        return 'Asset'

    def _get_property_filters(self) -> Mapping[str, "PropertyFilter[Property]"]:
        props = {
            'hostname': self._hostname,
        }

        return {p[0]: p[1] for p in props if p[1]}

    def _get_forward_edges(self) -> Mapping[str, "Queryable"]:
        return {}

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        return {}

    def with_hostname(
            self,
            eq: Optional[StrCmp] = None,
            contains: Optional[StrCmp] = None,
            ends_with: Optional[StrCmp] = None,
            starts_with: Optional[StrCmp] = None,
            regexp: Optional[StrCmp] = None,
            distance: Optional[Tuple[StrCmp, int]] = None,
    ) -> 'NQ':
        self._hostname.extend(
            _str_cmps(
                "hostname",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )

        return self


class AssetView(Viewable):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: str,
            hostname: Optional[str] = None,
    ):
        super(AssetView, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid
        )
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid

        self.hostname = hostname

    def get_node_type(self) -> str:
        return 'Asset'

    def get_hostname(self) -> Optional[str]:
        if not self.hostname:
            self.hostname = cast(Optional[str], self.fetch_property("hostname", str))
        return self.hostname

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            'hostname': str,
        }

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        return {}

    def _get_reverse_edges(self) -> "Mapping[str,  ReverseEdgeView]":
        return {}

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
            'hostname': self.hostname,
        }

        return {p[0]: p[1] for p in props.items() if p[1] is not None}
