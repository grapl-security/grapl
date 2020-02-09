import abc
import json
from typing import (
    Union,
    Mapping,
    TypeVar,
    Tuple,
    Dict,
    List,
    Type,
    Generic,
    Optional,
    Callable,
    Any,
    cast,
)

# noinspection Mypy
from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.comparators import unescape_dgraph_str
from grapl_analyzerlib.nodes.types import PropertyT, OneOrMany, Property

NV = TypeVar("NV", bound="Viewable")


class Viewable(abc.ABC):

    dynamic_forward_edge_types = {}  # type: Dict[str, EdgeViewT]
    dynamic_reverse_edge_types = {}  # type: Dict[str, Tuple[EdgeViewT, str]]
    dynamic_property_types = {}  # type: Dict[str, PropertyT]

    def __init__(
        self: "Viewable",
        dgraph_client: DgraphClient,
        node_key: str,
        uid: str,
        **args: Any,
    ) -> None:
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid

        self.dynamic_forward_edges = {}  # type: Dict[str, ForwardEdgeView]
        self.dynamic_reverse_edges = {}  # type: Dict[str,  ReverseEdgeView]
        self.dynamic_properties = {}  # type: Dict[str, Property]

    def extend(self, extended_type: Type[NV]) -> NV:
        return extended_type(self.dgraph_client, self.node_key, self.uid)

    @staticmethod
    @abc.abstractmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        pass

    @staticmethod
    @abc.abstractmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        pass

    @staticmethod
    @abc.abstractmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        """
        :return: Mapping of reverse edge name to Tuple of edge type and forward edge name
        """
        pass

    @abc.abstractmethod
    def _get_properties(self) -> Mapping[str, "Property"]:
        pass

    @abc.abstractmethod
    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        pass

    @abc.abstractmethod
    def _get_reverse_edges(self) -> "Mapping[str,  ReverseEdgeView]":
        pass

    @abc.abstractmethod
    def get_node_type(self) -> str:
        pass

    @classmethod
    def get_edge_types(
        cls
    ) -> Mapping[str, Union["EdgeViewT", Tuple["EdgeViewT", str]]]:
        return {
            **cls._get_forward_edge_types(),
            **cls.dynamic_forward_edge_types,
            **cls.dynamic_reverse_edge_types,
            **cls._get_reverse_edge_types(),
        }

    @classmethod
    def get_forward_edge_types(cls) -> Mapping[str, "EdgeViewT"]:
        return {**cls._get_forward_edge_types(), **cls.dynamic_forward_edge_types}

    @classmethod
    def get_reverse_edge_types(cls) -> Mapping[str, Tuple["EdgeViewT", str]]:
        return {**cls.dynamic_reverse_edge_types, **cls._get_reverse_edge_types()}

    def set_dynamic_property_type(self, prop_name: str, prop_type: "PropertyT") -> None:
        self.dynamic_property_types[prop_name] = prop_type

    @classmethod
    def set_dynamic_forward_edge_type(
        cls, edge_name: str, edge_type: "EdgeViewT"
    ) -> None:
        cls.dynamic_forward_edge_types[edge_name] = edge_type

    def set_dynamic_reverse_edge_type(
        self, edge_name: str, edge_type: "EdgeViewT", forward_name: str
    ) -> None:
        self.dynamic_reverse_edge_types[edge_name] = edge_type, forward_name

    def set_dynamic_property(self, prop_name: str, prop: "Property") -> None:
        self.dynamic_properties[prop_name] = prop

    def set_dynamic_forward_edge(self, edge_name: str, edge: "ForwardEdgeView") -> None:
        if edge:
            self.dynamic_forward_edges[edge_name] = edge

    def set_dynamic_reverse_edge(
        self, edge_name: str, reverse_edge: "ReverseEdgeView"
    ) -> None:
        self.dynamic_reverse_edges[edge_name] = reverse_edge

    def get_properties(self) -> Mapping[str, "Property"]:
        return cast(
            "Mapping[str, 'Property']",
            {
                **self._get_properties(),
                **self.dynamic_properties,
                "node_key": self.node_key,
                "uid": self.uid,
                "dgraph.type": self.get_node_type(),
            },
        )

    def get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        return {
            k: v
            for k, v in {
                **self._get_forward_edges(),
                **self.dynamic_forward_edges,
            }.items()
            if v
        }

    def get_reverse_edges(self) -> "Mapping[str,  ReverseEdgeView]":
        return {
            k: v
            for k, v in {
                **self._get_reverse_edges(),
                **self.dynamic_reverse_edges,
            }.items()
            if v[0]
        }

    def get_edges(self) -> "Mapping[str, EdgeView]":
        return {**self.get_forward_edges(), **self.get_reverse_edges()}

    def fetch_property(
        self, prop_name: str, prop_type: Callable[["Property"], "Property"]
    ) -> Optional[Union[str, int]]:
        node_key_prop = ""
        if prop_name != "node_key":
            node_key_prop = "node_key"
        query = f"""
            {{
                res(func: uid("{self.uid}"), first: 1) @cascade {{
                    uid,
                    {node_key_prop},
                    {prop_name}
                }}
            
            }}
        """

        txn = self.dgraph_client.txn(read_only=True)
        try:
            res = json.loads(txn.query(query).json)
        finally:
            txn.discard()
        raw_prop = res["res"]
        if not raw_prop:
            return None
        if isinstance(raw_prop, list):
            raw_prop = raw_prop[0]

        if raw_prop.get(prop_name) is None:
            return None

        prop = prop_type(raw_prop[prop_name])
        if isinstance(prop, str):
            return unescape_dgraph_str(prop)
        return prop

    def fetch_properties(
        self, prop_name: str, prop_type: Callable[["Property"], "Property"]
    ) -> List["Property"]:
        query = f"""
            {{
                res(func: uid("{self.uid}")) @cascade {{
                    uid,
                    node_key,
                    {prop_name}
                }}
            
            }}
        """
        txn = self.dgraph_client.txn(read_only=True)
        try:
            res = json.loads(txn.query(query).json)
        finally:
            txn.discard()
        raw_props = res["res"]

        if not raw_props:
            return []

        props = [prop_type(p[prop_name]) for p in raw_props]
        props = [unescape_dgraph_str(prop) for prop in props]
        return props

    def fetch_edge(self, edge_name: str, edge_type: Type["NV"]) -> Optional["NV"]:
        query = f"""
            {{
                res(func: uid("{self.uid}"), first: 1) {{
                    uid,
                    node_key,
                    {edge_name} {{
                        uid,
                        node_type: dgraph.type,
                        node_key,
                    }}
                }}
            
            }}
        """

        txn = self.dgraph_client.txn(read_only=True)
        try:
            res = json.loads(txn.query(query).json)
        finally:
            txn.discard()

        raw_edge = res["res"]
        if not raw_edge:
            return None

        if isinstance(raw_edge, list):
            raw_edge = raw_edge[0]

        if not raw_edge.get(edge_name):
            return None

        neighbor = raw_edge[edge_name]
        if isinstance(neighbor, list):
            neighbor = neighbor[0]

        edge = edge_type.from_dict(self.dgraph_client, neighbor)  # type: NV
        return edge

    def fetch_edges(self, edge_name: str, edge_type: Type["NV"]) -> List["NV"]:
        query = f"""
            {{
                res(func: uid("{self.uid}")) {{
                    uid,
                    node_key,
                    {edge_name} {{
                        uid,
                        node_type: dgraph.type,
                        node_key,
                    }}
                }}
            
            }}
        """
        txn = self.dgraph_client.txn(read_only=True)
        try:
            res = json.loads(txn.query(query).json)
        finally:
            txn.discard()

        raw_edges = res["res"]

        if not raw_edges or not raw_edges[0].get(edge_name):
            return []

        if isinstance(raw_edges, list):
            raw_edges = raw_edges[0][edge_name]
        else:
            raw_edges = raw_edges[edge_name]

        edges = [
            edge_type.from_dict(self.dgraph_client, f) for f in raw_edges
        ]  # type: List[NV]

        return edges

    @classmethod
    def from_dict(
        cls: Type["NV"], dgraph_client: DgraphClient, d: Dict[str, Any]
    ) -> "NV":
        properties = {}
        node_type = d.get("node_type") or d.get("dgraph.type")
        if node_type:
            if isinstance(node_type, list):
                if len(node_type) > 1:
                    print(
                        f"WARN: Node has multiple types: {node_type}, narrowing to: {node_type[0]}"
                    )
                node_type = node_type[0]
            properties["node_type"] = node_type
        else:
            print(f"WARN: Node is missing type: {d.get('node_key', d.get('uid'))}")

        for prop, into in cls._get_property_types().items():
            val = d.get(prop)

            if val or val == 0:
                if into == str:
                    val = unescape_dgraph_str(str(val))
                elif into == int:
                    val = int(val)
                properties[prop] = val

        edges = {}  # type: Dict[str, Union[Viewable, List[Viewable]]]
        for edge_tuple in cls.get_edge_types().items():
            edge_name = edge_tuple[0]  # type: str
            forward_name = None  # type: Optional[str]
            if isinstance(edge_tuple[1], tuple):
                ty = edge_tuple[1][0]  # type: EdgeViewT
                forward_name = edge_tuple[1][1]
            else:
                ty = edge_tuple[1]

            raw_edge = d.get(edge_name, None)

            if not raw_edge:
                continue

            if isinstance(ty, List):
                ty = ty[0]

                if d.get(edge_name, None):
                    f_edge = d[edge_name]
                    if isinstance(f_edge, list):
                        _edges = [ty.from_dict(dgraph_client, f) for f in f_edge]
                    elif isinstance(f_edge, dict):
                        _edges = [ty.from_dict(dgraph_client, f_edge)]
                    else:
                        raise TypeError(f"Edge {edge_name} must be list or dict")

                    if forward_name:
                        edges[forward_name] = _edges
                    else:
                        edges[edge_name] = _edges

            else:
                if isinstance(raw_edge, list):
                    edge = ty.from_dict(dgraph_client, raw_edge[0])
                elif isinstance(raw_edge, dict):
                    edge = ty.from_dict(dgraph_client, raw_edge)
                else:
                    raise TypeError(f"Edge {edge_name} must be list or dict")

                if forward_name:
                    edges[forward_name] = edge
                else:
                    edges[edge_name] = edge

        cleaned_edges = {}  # type: Dict[str, Union[Viewable, List[Viewable]]]
        for _edge in edges.items():
            edge_name = _edge[0]
            cleaned_edge = _edge[1]  # type: Union[Viewable, List[Viewable]]
            if edge_name[0] == "~":
                edge_name = edge_name[1:]
            cleaned_edges[edge_name] = cleaned_edge

        return cls(
            dgraph_client=dgraph_client,
            node_key=d["node_key"],
            uid=d["uid"],
            **properties,
            **cleaned_edges,
        )

    def to_dict(self) -> Dict[str, Any]:
        node_dict = dict(self.get_properties())
        node_dict["node_key"] = self.node_key
        edges = []

        for edge_name, edge in self.get_edges().items():
            # List of forward edges
            if isinstance(edge, list):
                for e in edge:
                    edges.append(
                        {
                            "from": self.node_key,
                            "edge_name": edge_name,
                            "to": e.node_key,
                        }
                    )
            # One or Many reverse edges
            elif isinstance(edge, tuple):
                # Many reverse edges
                if isinstance(edge[0], list):
                    for e in edge[0]:
                        edges.append(
                            {
                                "from": self.node_key,
                                "edge_name": edge_name,
                                "to": e.node_key,
                            }
                        )
                # One reverse edge
                else:
                    edges.append(
                        {
                            "from": self.node_key,
                            "edge_name": edge_name,
                            "to": edge[0].node_key,
                        }
                    )
            # One forward edge
            else:
                edges.append(
                    {"from": self.node_key, "edge_name": edge_name, "to": edge.node_key}
                )

        return {"node": node_dict, "edges": edges}


ForwardEdgeView = OneOrMany[Viewable]
ReverseEdgeView = Tuple[OneOrMany[Viewable], str]
EdgeView = Union[ForwardEdgeView, ReverseEdgeView]
EdgeViewT = Union[List[Type[Viewable]], Type[Viewable]]
