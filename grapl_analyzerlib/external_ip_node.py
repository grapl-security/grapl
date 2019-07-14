import json
from copy import deepcopy
from typing import List, Optional, Any, Tuple, Union, Dict, Set

from pydgraph import DgraphClient

from grapl_analyzerlib.node_types import PQ, FQ, OCQ, EIPQ, EIPV
from grapl_analyzerlib.querying import Has, Cmp, Queryable, Eq, _str_cmps, get_var_block, _generate_filter, _build_query, \
    _get_queries, Viewable, Not


class ExternalIpQuery(Queryable):
    def __init__(self) -> None:
        self._node_key = Has(
            "node_key"
        )  # type: Cmp
        self._uid = Has(
            "uid"
        )  # type: Cmp

        self._external_ip = []  # type: List[List[Cmp]]

        # Edges
        self._connections_from = None  # type: Optional[OCQ]

        # Meta
        self._first = None  # type: Optional[int]

    def with_external_ip(
            self,
            eq: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            contains: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            ends_with: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
    ) -> EIPQ:
        self._external_ip.extend(
            _str_cmps("external_ip", eq, contains, ends_with)
        )
        return self

    def with_connections_from(
            self,
            process: PQ
    ) -> EIPQ:
        process = deepcopy(process)
        process._created_connection = self
        self._connections_from = process
        return self

    def get_properties(self) -> List[str]:
        properties = (
            "node_key" if self._node_key else None,
            "uid" if self._uid else None,
            "external_ip" if self._external_ip else None,
        )

        return [p for p in properties if p]

    def get_neighbors(self) -> List[Any]:
        neighbors = (self._connections_from,)

        return [n for n in neighbors if n]

    def get_edges(self) -> List[Tuple[str, Any]]:
        neighbors = (
            ("connections_from", self._connections_from) if self._connections_from else None,
        )

        return [n for n in neighbors if n]

    def _get_var_block(
            self, binding_num: int, root: Any, already_converted: Set[Any]
    ) -> str:
        if self in already_converted:
            return ""
        already_converted.add(self)

        filters = self._filters()

        connections_from = get_var_block(
            self._connections_from, "~external_connections", binding_num, root, already_converted
        )

        block = f"""
            {filters} {{
                {connections_from}
            }}
            """

        return block

    def _get_var_block_root(
            self, binding_num: int, root: Any, node_key: Optional[str] = None
    ) -> str:
        already_converted = {self}
        root_var = ""
        if self == root:
            root_var = f"Binding{binding_num} as "

        filters = self._filters()

        connections_from = get_var_block(
            self._connections_from, "~external_connections", binding_num, root, already_converted
        )

        func_filter = """has(external_ip)"""
        if node_key:
            func_filter = f'eq(node_key, "{node_key}")'

        block = f"""
            {root_var} var(func: {func_filter}) @cascade {filters} {{
                {connections_from}
            }}
            """

        return block

    def _filters(self) -> str:
        inner_filters = (
            _generate_filter(self._connections_from),
        )

        inner_filters = [i for i in inner_filters if i]
        if not inner_filters:
            return ""
        return f"@filter({'AND'.join(inner_filters)})"



class ExternalIpView(Viewable):
    def __init__(self, dgraph_client: DgraphClient, node_key: str, uid: Optional[str] = None,
                 external_ip: Optional[str] = None) -> None:
        super(ExternalIpView, self).__init__(self)
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.external_ip = external_ip

    @staticmethod
    def from_dict(dgraph_client: DgraphClient, d: Dict[str, Any]) -> EIPV:

        return ExternalIpView(
            dgraph_client=dgraph_client,
            node_key=d['node_key'],
            uid=d['uid'],
            external_ip=d.get('external_ip', None),
        )


