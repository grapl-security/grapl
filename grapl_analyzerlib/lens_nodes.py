import json
from typing import Any, Dict, List, Tuple, Optional, Union, Type, Callable

from pydgraph import DgraphClient, Txn

from grapl_analyzerlib.entities import NodeView, ProcessView, PV, ProcessQuery
from grapl_analyzerlib.querying import Queryable, Viewable, PropertyFilter, V, StrCmp, Cmp, _str_cmps, Has, Eq, Not


def stripped_node_to_query(node: Dict[str, Union[str, int]]) -> str:
    func_filter = f'eq(node_key, "{node["node_key"]}")'
    return f"""
        {{
            res(func: {func_filter}, first: 1) {{
                uid,
                node_key,
            }}
        }}
    """


def get_edges(node) -> List[Tuple[str, str, str]]:
    edges = []

    for key, value in node.items():
        if isinstance(value, dict):
            edges.append(
                (node['uid'], key, value['uid'])
            )
        elif isinstance(value, list):
            for neighbor in value:
                edges.append(
                    (node['uid'], key, neighbor['uid'])
                )

    return edges


def strip_node(node) -> Dict[str, Any]:
    output = {}
    for key, value in node.items():
        if isinstance(value, str) or isinstance(value, int):
            output[key] = value
    return output


def mut_from_response(res, nodes, edges):
    if isinstance(res, dict):
        edges.extend(get_edges(res))
        nodes.append(res)
        for element in res.values():
            if type(element) is list:
                mut_from_response(element, nodes, edges)
            if type(element) is dict:
                mut_from_response(element, nodes, edges)
    else:
        for element in res:
            if type(element) is list:
                mut_from_response(element, nodes, edges)
            if type(element) is dict:
                mut_from_response(element, nodes, edges)


class CopyingTransaction(Txn):
    def __init__(self, copying_client, read_only=False, best_effort=False) -> None:
        super().__init__(copying_client.src_client, read_only, best_effort)
        self.src_client = copying_client.src_client
        self.dst_client = copying_client.dst_client
        self.copied_uids = []

    def get_copied_uids(self) -> List[str]:
        return self.copied_uids

    def query(self, query, variables=None, timeout=None, metadata=None,
              credentials=None):
        """
        Query the dst graph.
        if response, return response
        If it does not, check if it exists in src graph
        if it does
            * copy from src graph to dst graph
            * hook up new nodes to the engagement
        return query on dst graph
        :return:
        """

        # Query dst_graph
        dst_txn = self.dst_client.txn(read_only=True, best_effort=False)  # type: Txn
        try:
            res = dst_txn.query(query, variables, timeout, metadata, credentials)
            _res = json.loads(res.json)
            # If any query has values, return res
            for response in _res.values():
                if response:
                    nodes = []
                    for v in _res.values():
                        mut_from_response(v, nodes, [])
                    self.copied_uids = [node['uid'] for node in nodes]
                    return res
        finally:
            dst_txn.discard()

        # Otherwise, try to copy from src to dst
        # Query source
        txn = self.src_client.txn(read_only=True)
        try:
            res = (
                txn.query(query, variables, timeout, metadata, credentials)
            )
        finally:
            txn.discard()
        # If it isn't in the source, return the empty response
        _res = json.loads(res.json)

        if not any(_res.values()):
            return res

        # Otherwise, mutate the dst graph with the response
        nodes = []
        edges = []
        for v in _res.values():
            mut_from_response(v, nodes, edges)

        uid_map = {}
        nodes = [(node.pop('uid'), strip_node(node)) for node in nodes]

        for old_uid, stripped_node in nodes:
            query = stripped_node_to_query(stripped_node)

            try:
                dst_txn = self.dst_client.txn(read_only=False, best_effort=False)  # type: Txn

                _txn = self.dst_client.txn(read_only=False)
                try:
                    res = (
                        _txn.query(query, variables, timeout, metadata, credentials)
                    ).json
                finally:
                    _txn.discard()

                res = json.loads(res)['res']

                new_uid = None
                if res:
                    stripped_node['uid'] = res[0]['uid']
                    new_uid = res[0]['uid']

                m_res = dst_txn.mutate(set_obj=stripped_node, commit_now=True)
                uids = m_res.uids

                new_uid = new_uid or uids['blank-0']
                uid_map[old_uid] = new_uid

                self.copied_uids.append(new_uid)

            finally:
                dst_txn.discard()

        for from_edge, edge_name, to_edge in edges:
            if edge_name[0] == '~':
                edge_name = edge_name[1:]
                mu = {
                    'uid': uid_map[to_edge],
                    edge_name: {
                        'uid': uid_map[from_edge]
                    }
                }

            else:
                mu = {
                    'uid': uid_map[from_edge],
                    edge_name: {
                        'uid': uid_map[to_edge]
                    }
                }

            dst_txn = self.dst_client.txn(read_only=False)
            dst_txn.mutate(set_obj=mu, commit_now=True)

        # Query dst_graph again
        txn = super()
        try:
            qr = txn.query(query, variables, timeout, metadata, credentials)
        finally:
            txn.discard()
        return qr

class CopyingDgraphClient(DgraphClient):
    def __init__(self, src_client: DgraphClient, dst_client: DgraphClient) -> None:
        super().__init__(*src_client._clients, *dst_client._clients)
        self.src_client = src_client
        self.dst_client = dst_client

    def txn(self, read_only=False, best_effort=False) -> CopyingTransaction:
        return CopyingTransaction(self, read_only=read_only, best_effort=best_effort)


class EngagementTransaction(CopyingTransaction):
    def __init__(self, copying_client, eg_uid: str, read_only=False, best_effort=False) -> None:
        super().__init__(copying_client, read_only=read_only, best_effort=best_effort)
        self.eg_uid = eg_uid

    def query(self, query, variables=None, timeout=None, metadata=None,
              credentials=None):
        txn = super()
        res = txn.query(query, variables, timeout, metadata, credentials)

        for uid in txn.get_copied_uids():
            if uid == self.eg_uid:
                continue
            dst_txn = self.dst_client.txn(read_only=False)
            try:
                mu = {
                    'uid': self.eg_uid,
                    'scope': {
                        'uid': uid
                    }
                }

                dst_txn.mutate(set_obj=mu, commit_now=True)
            finally:
                dst_txn.discard()
        return res


class EngagementClient(CopyingDgraphClient):
    def __init__(
            self,
            eg_uid: str,
            src_client: DgraphClient,
            dst_client: DgraphClient,
    ):
        super().__init__(src_client, dst_client)
        self.eg_uid = eg_uid

    def txn(self, read_only=False, best_effort=False) -> CopyingTransaction:
        return EngagementTransaction(self, self.eg_uid, read_only=read_only, best_effort=best_effort)


class EngagementQuery(Queryable):
    def __init__(self):
        super(EngagementQuery, self).__init__(EngagementView)
        self._name = []  # type: List[List[Cmp]]
        self._scope = None  # type: Optional[Queryable]

    def with_node_key(self, eq: Optional[Union[Not, str]] = None):
        if eq:
            self._node_key = Eq("node_key", eq)
        else:
            self._node_key = Has("node_key")
        return self

    def get_unique_predicate(self) -> Optional[str]:
        return 'lens'

    def get_node_type_name(self) -> Optional[str]:
        return None

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        if not self._uid:
            return []
        return [[self._uid]]

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        props = [
            ("node_key", self.get_node_key_filter()),
            ('lens', self._name),
        ]
        return [p for p in props if p[1]]

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        edges = [('scope', self._scope)]
        return [e for e in edges if e[1]]

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        return []

    def with_name(self, eq=StrCmp) -> 'EngagementQuery':
        self._name.extend(_str_cmps("lens", eq, None, None))
        return self


class EngagementView(Viewable):
    def __init__(
            self,
            dgraph_client: EngagementClient,
            node_key: str,
            uid: str,
            lens: str,
            scope: Optional[List[NodeView]] = None,
            **kwargs,
    ):
        super().__init__(dgraph_client, node_key, uid)
        self.lens = lens
        self.engagement_client = dgraph_client
        self.scope = scope or []

    @staticmethod
    def get_property_types() -> List[Tuple[str, Callable[[Any], Union[str, int]]]]:
        return [('lens', str)]

    @staticmethod
    def get_edge_types() -> List[Tuple[str, Union[List[Type[V]], Type[V]]]]:
        return [('scope', [NodeView])]

    def get_property_tuples(self) -> List[Tuple[str, Any]]:
        props = [

        ]

        return [p for p in props if p[1]]

    def get_edge_tuples(self) -> List[Tuple[str, Any]]:
        edges = [

        ]

        return [e for e in edges if e[1]]

    @staticmethod
    def get_or_create(name: str, copy_client: CopyingDgraphClient) -> 'EngagementView':
        eg_graph = copy_client.dst_client

        eg_txn = eg_graph.txn(read_only=False)
        try:
            query = """
            query res($a: string)
            {
              res(func: eq(lens, $a), first: 1) @cascade
               {
                 uid,
                 node_key,
               }
             }"""
            res = eg_txn.query(
                query, variables={'$a': name}
            )

            res = json.loads(res.json)['res']
            new_uid = None
            if res:
                new_uid = res[0]['uid']
            else:
                m_res = eg_txn.mutate(
                    set_obj={
                        "lens": name,
                        "node_key": name,
                        "score": 0,
                    }, commit_now=True)
                uids = m_res.uids

                new_uid = new_uid or uids['blank-0']
        finally:
            eg_txn.discard()

        engagement_client = EngagementClient(
            new_uid,
            copy_client.src_client,
            copy_client.dst_client
        )

        return EngagementQuery().with_name(eq=name).query_first(engagement_client)

    def remove_node(self, node_key: str):
        node_uid = None
        for ix, node in enumerate(self.scope):
            if node.get_node_key() == node_key:
                self.scope.pop(ix)
                node_uid = node.uid

        if not node_uid:
            node_uid = (
                ProcessQuery()
                .with_node_key(node_key)
                .query_first(self.engagement_client.dst_client)
            )

            if not node_uid:
                return
            node_uid = node_uid.uid
        # Remove edge between engagement and node
        txn = self.engagement_client.dst_client.txn(read_only=False)
        try:
            txn.mutate(
                del_obj={
                    "uid": self.uid,
                    "scope": {
                        "uid": node_uid
                    }
                }, commit_now=True)
        finally:
            txn.discard()

        txn = self.engagement_client.dst_client.txn(read_only=False)
        try:
            query = """
            query res($a: string)
            {
              res(func: uid($a), first: 1) @cascade
               {
                ~scope {
                    uid
                }
               }
             }"""
            res = txn.query(
                query, variables={'$a': node_uid}
            )
            res = json.loads(res.json)['res']

            if not res:
                txn.mutate(
                    del_obj={'uid': node_uid},
                    commit_now=True
                )

        finally:
            txn.discard()

        # If node is not a part of any scope, remove it

    def get_process(self, node_key: str, copy=True) -> Optional['PV']:
        for node in self.scope:
            if node.get_node_key() == node_key:
                return node.as_process_view()

        if copy:
            client = self.engagement_client
        else:
            client = self.engagement_client.dst_client

        p = (
            ProcessQuery()
                .with_node_key(node_key)
                .with_process_id()
                .with_process_name()
                .query_first(client)
        )  # type: Optional[ProcessView]

        if not p:
            return None

        self.scope.append(
            NodeView(
                self.engagement_client,
                node_key,
                p.uid,
                p
            )
        )

        return p

    def get_node(self, node_key: str, copy=True) -> Optional['NodeView']:
        for node in self.scope:
            if node.get_node_key() == node_key:
                return node

        if copy:
            client = self.engagement_client
        else:
            client = self.engagement_client.dst_client

        node = NodeView.from_node_key(client, node_key)

        if not node:
            return None

        self.scope.append(node)

        return node
