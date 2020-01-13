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
                    self.copied_uids = [node["uid"] for node in nodes]
                    return res
        finally:
            dst_txn.discard()

        # Otherwise, try to copy from src to dst
        # Query source
        txn = self.src_client.txn(read_only=True)
        try:
            res = txn.query(query, variables, timeout, metadata, credentials)
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
        nodes = [(node.pop("uid"), strip_node(node)) for node in nodes]

        for old_uid, stripped_node in nodes:
            query = stripped_node_to_query(stripped_node)

            try:
                dst_txn = self.dst_client.txn(
                    read_only=False, best_effort=False
                )  # type: Txn

                _txn = self.dst_client.txn(read_only=False)
                try:
                    res = (
                        _txn.query(query, variables, timeout, metadata, credentials)
                    ).json
                finally:
                    _txn.discard()

                res = json.loads(res)["res"]

                new_uid = None
                if res:
                    stripped_node["uid"] = res[0]["uid"]
                    new_uid = res[0]["uid"]

                m_res = dst_txn.mutate(set_obj=stripped_node, commit_now=True)
                uids = m_res.uids

                new_uid = new_uid or uids["blank-0"]
                uid_map[old_uid] = new_uid

                self.copied_uids.append(new_uid)

            finally:
                dst_txn.discard()

        for from_edge, edge_name, to_edge in edges:
            if edge_name[0] == "~":
                edge_name = edge_name[1:]
                mu = {"uid": uid_map[to_edge], edge_name: {"uid": uid_map[from_edge]}}

            else:
                mu = {"uid": uid_map[from_edge], edge_name: {"uid": uid_map[to_edge]}}

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
    def __init__(
        self, copying_client, eg_uid: str, read_only=False, best_effort=False
    ) -> None:
        super().__init__(copying_client, read_only=read_only, best_effort=best_effort)
        self.eg_uid = eg_uid

    def query(
        self, query, variables=None, timeout=None, metadata=None, credentials=None
    ):
        txn = super()
        res = txn.query(query, variables, timeout, metadata, credentials)

        for uid in txn.get_copied_uids():
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

    def txn(self, read_only=False, best_effort=False) -> CopyingTransaction:
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

        engagement_client = EngagementClient(
            new_uid, copy_client.src_client, copy_client.dst_client
        )

        self_lens = (
            LensQuery().with_lens_name(eq=lens_name).query_first(engagement_client)
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


from grapl_analyzerlib.nodes.comparators import PropertyFilter, Cmp, StrCmp, _str_cmps
from grapl_analyzerlib.prelude import NodeView
