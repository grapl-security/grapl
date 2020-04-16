import json
from typing import *

# noinspection Mypy
from pydgraph import DgraphClient, Txn

from grapl_analyzerlib.nodes.queryable import Queryable
from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.nodes.viewable import (
    Viewable,
    EdgeViewT,
    ForwardEdgeView,
    ReverseEdgeView,
)

T = TypeVar("T")

ILensQuery = TypeVar("ILensQuery", bound="LensQuery")
ILensView = TypeVar("ILensView", bound="LensView")


def stripped_node_to_query(node: Dict[str, Union[str, int]]) -> str:
    func_filter = f'eq(node_key, "{node["node_key"]}")'
    return f"""
        {{
            res(func: {func_filter}, first: 1) {{
                uid,
                node_key,
                dgraph.type: node_type,
            }}
        }}
    """


def get_edges(node: Dict[str, Any]) -> List[Tuple[str, str, str]]:
    edges = []

    for key, value in node.items():
        if isinstance(value, dict):
            edges.append((node["uid"], key, value["uid"]))
        elif isinstance(value, list):
            for neighbor in value:
                if isinstance(neighbor, dict):
                    edges.append((node["uid"], key, neighbor["uid"]))

    return edges


def strip_node(node) -> Dict[str, Any]:
    output = {}
    for key, value in node.items():
        if key == 'node_type' or key == 'dgraph.type':
            output['dgraph.type'] = value
        if isinstance(value, str) or isinstance(value, int):
            output[key] = value
    return output


def response_into_matrix(res, nodes, edges):
    if isinstance(res, dict):
        edges.extend(get_edges(res))
        nodes[res['node_key']] = strip_node(res)
        for element in res.values():
            if type(element) is list:
                response_into_matrix(element, nodes, edges)
            if type(element) is dict:
                response_into_matrix(element, nodes, edges)
    else:
        for element in res:
            if type(element) is list:
                response_into_matrix(element, nodes, edges)
            if type(element) is dict:
                response_into_matrix(element, nodes, edges)


def dg_query(client: DgraphClient, query: str) -> Dict[str, Any]:
    txn = client.txn(read_only=True, best_effort=False)  # type: Txn
    try:
        return txn.query(query, variables=None, timeout=None, metadata=None, credentials=None)
    finally:
        txn.discard()


def upsert(client: DgraphClient, node_dict: Dict[str, Any]) -> None:
    if node_dict.get('uid'):
        node_dict.pop('uid')
    node_dict['uid'] = '_:blank-0'
    node_key = node_dict['node_key']
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}"))  @cascade {{
                    uid,
                    dgraph.type,
            }}
        }}
        """
    txn = client.txn(read_only=False)

    try:
        res = json.loads(txn.query(query).json)['q0']

        if res:
            node_dict['uid'] = res[0]['uid']
            node_dict = {**node_dict, **res[0]}

        mutation = node_dict

        mut_res = txn.mutate(set_obj=mutation, commit_now=True)
        new_uid = node_dict.get('uid') or mut_res.uids["blank-0"]
        return new_uid

    finally:
        txn.discard()


def copy_node(
        src_client: DgraphClient,
        dst_client: DgraphClient,
        node_key: str,
        init_node: Optional[Dict[str, Any]] = None
) -> None:
    if not init_node:
        init_node = dict()
    assert init_node is not None

    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}")) {{
                    uid,
                    dgraph.type,
                    expand(_all_),
            }}
        }}
        """
    txn = src_client.txn(read_only=True)

    try:
        res = json.loads(txn.query(query).json)['q0']
    finally:
        txn.discard()

    if not res:
        raise Exception("ERROR: Can not find res")
    print('res', res)
    if not res[0].get('dgraph.type'):
        raise Exception("ERROR: Can not find res dgraph.type")

    raw_to_copy = {**res[0], **init_node}

    return upsert(dst_client, raw_to_copy)


def create_edge(client: DgraphClient, from_uid: str, edge_name: str, to_uid: str) -> None:
    if edge_name[0] == '~':
        mut = {
            'uid': to_uid,
            edge_name[1:]: {'uid': from_uid}
        }

    else:
        mut = {
            'uid': from_uid,
            edge_name: {'uid': to_uid}
        }

    txn = client.txn(read_only=False)
    try:
        res = txn.mutate(set_obj=mut, commit_now=True)
    finally:
        txn.discard()


class CopyingTransaction(Txn):
    def __init__(self, copying_client, read_only=False, best_effort=False) -> None:
        super().__init__(copying_client.src_client, read_only, best_effort)
        self.src_client = copying_client.src_client
        self.dst_client = copying_client.dst_client
        self.copied_uids = []  # type: List[str]

    def get_copied_uids(self) -> List[str]:
        return self.copied_uids

    def query(
            self, query, variables=None, timeout=None, metadata=None, credentials=None
    ):
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

        # Query the dst graph
        raw_dst_res = dg_query(self.dst_client, query)
        dst_res = json.loads(raw_dst_res.json)
        # If it's already there, return the response, no need to copy
        if any(dst_res.values()):
            # Make sure we register these nodes as 'copied', as they need to be tagged by our
            # engagement
            nodes = {}  # type: Dict[str, Dict[str, Any]]
            edges = []
            response_into_matrix(dst_res.values(), nodes, edges)
            for node in nodes.values():
                self.copied_uids.append(node['uid'])
            return raw_dst_res

        # Otherwise, we need to copy data from our src to dst
        # First we query the src graph
        # If we don't get a response, return
        raw_src_res = dg_query(self.src_client, query)
        src_res = json.loads(raw_src_res.json)
        if not any(src_res.values()):
            return raw_src_res

        # Next we strip the response from the src, removing any 'uid' values, since those
        # won't match up with our dst graph's uids
        # The end result of our mutation should be an adjacency matrix of nodes and edges

        nodes = {}  # type: Dict[str, Dict[str, Any]]
        edges = []
        response_into_matrix(src_res.values(), nodes, edges)
        uid_map = {}  # Map from old uid to new uid
        # Upsert every node, and write every edge
        for node in nodes.values():
            old_uid = node.pop('uid')
            new_uid = copy_node(
                self.src_client,
                self.dst_client,
                node['node_key'],
                node,
            )

            uid_map[old_uid] = new_uid


        self.copied_uids.extend(uid_map.values())

        for edge in edges:
            from_uid, edge_name, to_uid = edge
            create_edge(self.dst_client, uid_map[from_uid], edge_name, uid_map[to_uid])

        return dg_query(self.dst_client, query)


class CopyingDgraphClient(DgraphClient):
    def __init__(self, src_client: DgraphClient, dst_client: DgraphClient) -> None:
        super().__init__(*src_client._clients, *dst_client._clients)
        self.src_client = src_client
        self.dst_client = dst_client

    def txn(self, read_only=False, best_effort=False) -> CopyingTransaction:
        return CopyingTransaction(self, read_only=read_only, best_effort=best_effort)


class EngagementTransaction(CopyingTransaction):
    def __init__(
            self, copying_client, eg_uid: str, read_only=False, best_effort=False
    ) -> None:
        super().__init__(copying_client, read_only=read_only, best_effort=best_effort)
        self.eg_uid = eg_uid

    def query(
            self, query, variables=None, timeout=None, metadata=None, credentials=None
    ):
        copied_uids = set()

        # txn = super()
        # try:
        #     txn.query(query, variables, timeout, metadata, credentials)
        #     copied_uids.update(txn.get_copied_uids())
        # finally:
        #     txn.discard()
        # TODO: This is a hack. For some reason it doesn't copy edges properly the first time
        txn = super()
        try:
            res = txn.query(query, variables, timeout, metadata, credentials)
            copied_uids.update(txn.get_copied_uids())
        finally:
            txn.discard()

        for uid in copied_uids:
            if uid == self.eg_uid:
                continue
            dst_txn = self.dst_client.txn(read_only=False)
            try:
                mu = {"uid": self.eg_uid, "scope": {"uid": uid}}

                dst_txn.mutate(set_obj=mu, commit_now=True)
            finally:
                dst_txn.discard()
        return res


class EngagementClient(CopyingDgraphClient):
    def __init__(self, eg_uid: str, src_client: DgraphClient, dst_client: DgraphClient):
        super().__init__(src_client, dst_client)
        self.eg_uid = eg_uid

    @staticmethod
    def from_name(engagement_name: str, src_client: DgraphClient, dst_client: DgraphClient):
        cclient = CopyingDgraphClient(src_client=src_client, dst_client=dst_client)

        engagement_lens = LensView.get_or_create(cclient, engagement_name)
        return EngagementClient(engagement_lens.uid, src_client, dst_client)

    def txn(self, read_only=False, best_effort=False) -> EngagementTransaction:
        return EngagementTransaction(
            self, self.eg_uid, read_only=read_only, best_effort=best_effort
        )


class LensQuery(Queryable["LensView"]):
    def __init__(self) -> None:

        super(LensQuery, self).__init__(LensView)
        self._lens = []  # type: List[List[Cmp[str]]]

    def with_lens_name(
            self,
            eq: Optional["StrCmp"] = None,
            contains: Optional["StrCmp"] = None,
            ends_with: Optional["StrCmp"] = None,
    ) -> "LensQuery":
        self._lens.extend(_str_cmps("lens", eq, contains, ends_with))
        return self

    def _get_unique_predicate(self) -> Optional[Tuple[str, PropertyT]]:
        return "lens", int

    def _get_node_type_name(self) -> str:
        return 'Lens'

    def _get_property_filters(self) -> Mapping[str, "PropertyFilter[Property]"]:
        props = {"lens": self._lens}
        combined = {}
        for prop_name, prop_filter in props.items():
            if prop_filter:
                combined[prop_name] = cast("PropertyFilter[Property]", prop_filter)

        return combined

    def _get_forward_edges(self) -> Mapping[str, "Queryable"]:
        return {}

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        return {}


class LensView(Viewable):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            uid: str,
            node_key: str,
            node_type: Optional[str] = None,
            lens: Optional[str] = None,
            scope: Optional[List["NodeView"]] = None,
    ) -> None:
        super(LensView, self).__init__(dgraph_client, node_key=node_key, uid=uid)
        self.lens = lens
        self.node_type = node_type
        self.scope = scope or []

    def get_node_type(self) -> str:
        return 'Lens'

    @staticmethod
    def get_or_create(copy_client: CopyingDgraphClient, lens_name: str) -> "LensView":
        eg_txn = copy_client.dst_client.txn(read_only=False)
        try:
            query = """
            query res($a: string)
            {
              res(func: eq(lens, $a), first: 1) @cascade
               {
                 uid,
                 node_type: dgraph.type,
                 node_key,
               }
             }"""
            res = eg_txn.query(query, variables={"$a": lens_name})

            res = json.loads(res.json)["res"]
            new_uid = None
            if res:
                new_uid = res[0]["uid"]
            else:
                m_res = eg_txn.mutate(
                    set_obj={
                        "lens": lens_name,
                        "node_key": lens_name,
                        "dgraph.type": "Lens",
                        "score": 0,
                    },
                    commit_now=True,
                )
                uids = m_res.uids

                new_uid = new_uid or uids["blank-0"]
        finally:
            eg_txn.discard()

        self_lens = (
            LensQuery().with_lens_name(eq=lens_name).query_first(copy_client.dst_client)
        )
        assert self_lens, "Lens must exist"
        return self_lens

    def get_lens_name(self) -> Optional[str]:
        if self.lens is not None:
            return self.lens
        self.lens = cast(str, self.fetch_property("lens", str))
        return self.lens

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {"lens": str}

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        return {"scope": [NodeView]}

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        return {}

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        # TODO: Fetch it `fetch`
        _props = {"lens": self.lens}

        props = {
            p[0]: p[1] for p in _props.items() if p[1] is not None
        }  # type: Mapping[str, Union[str, int]]

        return props

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {"scope": self.scope}

        forward_edges = {
            name: value for name, value in f_edges.items() if value is not None
        }
        return cast("Mapping[str, ForwardEdgeView]", forward_edges)

    def _get_reverse_edges(self) -> "Mapping[str, ReverseEdgeView]":
        return {}


class EngagementView(LensView):

    @staticmethod
    def get_or_create(copy_client: CopyingDgraphClient, lens_name: str) -> "EngagementView":
        lens = LensView.get_or_create(copy_client, lens_name)

        engagement_client = EngagementClient(
            lens.uid,
            copy_client.src_client,
            copy_client.dst_client,
        )

        return EngagementView(engagement_client, lens.uid, lens.node_key)

    def get_node(self, node_key: str) -> Optional["NodeView"]:
        return NodeQuery().with_node_key(eq=node_key).query_first(self.dgraph_client)


from grapl_analyzerlib.nodes.comparators import PropertyFilter, Cmp, StrCmp, _str_cmps
from grapl_analyzerlib.prelude import NodeView, NodeQuery
