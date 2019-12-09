from typing import *

# noinspection Mypy
from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.queryable import Queryable
from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.nodes.viewable import Viewable, EdgeViewT, ForwardEdgeView, ReverseEdgeView

T = TypeVar("T")

IExternalIpQuery = TypeVar('IExternalIpQuery', bound='ExternalIpQuery')
IExternalIpView = TypeVar('IExternalIpView', bound='ExternalIpView')


class ExternalIpQuery(Queryable['ExternalIpView']):
    def __init__(self) -> None:
        super(ExternalIpQuery, self).__init__(ExternalIpView)
        self._external_ip = []  # type: List[List[Cmp[str]]]

        self._connections_from = None  # type: Optional['ProcessQuery']

    def _get_unique_predicate(self) -> Optional[Tuple[str, 'PropertyT']]:
        return 'external_ip', str

    def _get_node_type_name(self) -> Optional[str]:
        return None

    def _get_property_filters(self) -> 'Mapping[str, PropertyFilter[Property]]':
        _pfs = {
            'external_ip': self._external_ip
        }

        pfs = {p[0]: p[1] for p in _pfs.items() if p[1]}

        return cast('Mapping[str, PropertyFilter[Property]]', pfs)

    def _get_forward_edges(self) -> Mapping[str, "Queryable"]:
        return {}

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        reverse_edges = {
            "~created_connections": (self._connections_from, "connections_from")
        }

        return {fe[0]: (fe[1][0], fe[1][1]) for fe in reverse_edges.items() if fe[1][0] is not None}


class ExternalIpView(Viewable):

    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: str,
            node_type: Optional[str] = None,
            external_ip: Optional[str] = None,
            connections_from: Optional[List['ProcessView']] = None
    ):
        super(ExternalIpView, self).__init__(dgraph_client, node_key, uid)
        self.external_ip = external_ip
        self.node_type = node_type
        self.connections_from = connections_from or []

    def get_external_ip(self) -> Optional[str]:
        if self.external_ip is not None:
            return self.external_ip
        self.external_ip = cast(Optional[str], self.fetch_property('external_ip', str))
        return self.external_ip

    def get_connections_from(self) -> List['ProcessView']:
        return cast(List['ProcessView'], self.fetch_edges('~connections_from', ProcessView))

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            'external_ip': str,
        }

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        return {}

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        return {
            '~created_connections': ([ProcessView], 'connections_from')
        }

    def _get_properties(self) -> Mapping[str, 'Property']:
        _props = {
            'external_ip': self.external_ip,
        }

        props = {p[0]: p[1] for p in _props.items() if p[1] is not None}  # type: Mapping[str, Union[str, int]]

        return props


    def _get_forward_edges(self) -> 'Mapping[str, ForwardEdgeView]':
        pass

    def _get_reverse_edges(self) -> 'Mapping[str,  ReverseEdgeView]':
        _reverse_edges = {
            '~created_connections': (self.connections_from, 'connections_from')
        }

        reverse_edges = {name: value for name, value in _reverse_edges.items() if value[0] is not None}
        return cast(Mapping[str, ReverseEdgeView], reverse_edges)


from grapl_analyzerlib.nodes.process_node import ProcessView, ProcessQuery
from grapl_analyzerlib.nodes.comparators import PropertyFilter, Cmp, StrCmp, _str_cmps, IntCmp, _int_cmps
