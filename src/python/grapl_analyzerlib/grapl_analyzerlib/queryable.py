from __future__ import annotations
import abc
import functools
import json
from collections import defaultdict
from typing import (
    Any,
    cast,
    Dict,
    TypeVar,
    Generic,
    Type,
    Tuple,
    Optional,
    List,
    Union,
    TYPE_CHECKING,
)
from uuid import uuid4

from grapl_analyzerlib.comparators import (
    _str_cmps,
    _int_cmps,
    StrOrNot,
    OneOrMany,
    IntOrNot,
)
from grapl_analyzerlib.extendable import Extendable
from grapl_analyzerlib.grapl_client import GraphClient

if TYPE_CHECKING:
    from grapl_analyzerlib.viewable import Viewable

Q = TypeVar("Q", bound="Queryable")
V = TypeVar("V", bound="Viewable")
F = TypeVar("F", bound="Queryable")

ToOneFilter = List[F]
ToManyFilter = List[Tuple[F, ...]]
EdgeFilter = Union[ToOneFilter[F], ToManyFilter[F]]


def with_str_prop(prop):
    @functools.wraps(prop)
    def _with_str_prop(func):
        @functools.wraps(func)
        def wrapper_with_str_prop(self, **kwargs):
            return self.with_str_property(prop, **kwargs)

        return wrapper_with_str_prop

    return _with_str_prop


def with_int_prop(prop):
    @functools.wraps(prop)
    def _with_int_prop(func):
        @functools.wraps(func)
        def wrapper_with_int_prop(self, **kwargs):
            return self.with_int_property(prop, **kwargs)

        return wrapper_with_int_prop

    return _with_int_prop


def with_to_neighbor(*args):
    @functools.wraps(args)
    def _with_to_neighbor(func):
        @functools.wraps(func)
        def wrapper_with_to_neighbor(self, *edges):
            default = args[0] or type(self)
            f = args[1]
            r = args[2]
            return self.with_to_neighbor(default, f, r, edges)

        return wrapper_with_to_neighbor

    return _with_to_neighbor


class QueryFailedException(Exception):
    def __init__(self, query: Queryable, variables: Dict[VarPlaceholder, str]) -> None:
        super(QueryFailedException, self).__init__(
            "Failed query input\n" f"  Query: {query}\n" f"  Variables: {variables}\n"
        )


class Queryable(Generic[V, Q], Extendable, abc.ABC):
    def __init__(self):
        self._property_filters: Dict[str, List[List[Cmp]]] = defaultdict(list)
        self._edge_filters: Dict[str, EdgeFilter[Q]] = defaultdict(list)
        self._id = str(uuid4())

        for prop in type(self).node_schema().get_properties().keys():
            self.set_property_filters(prop, [])

        for f, (_t, r) in type(self).node_schema().get_edges().items():
            self.set_property_filters(f, [])
            self.set_property_filters(r, [])

    def with_node_key(self: Q, *, eq: str) -> Q:
        self._property_filters["node_key"] = [[Eq("node_key", eq)]]
        return self

    def with_to_neighbor(self, default, f, r, edges) -> "Q":
        if edges and not isinstance(edges, tuple):
            edges = (edges,)
        edges = edges or [default()]
        self.set_neighbor_filters(f, [edges])
        for edge in edges:
            edge.set_neighbor_filters(r, [self])
        return self

    def with_str_property(
        self,
        property_name: str,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        self._property_filters[property_name].extend(
            _str_cmps(
                predicate=property_name,
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_int_property(
        self,
        property_name: str,
        *,
        eq: Optional["IntOrNot"] = None,
        gt: Optional["IntOrNot"] = None,
        ge: Optional["IntOrNot"] = None,
        lt: Optional["IntOrNot"] = None,
        le: Optional["IntOrNot"] = None,
    ):
        self._property_filters[property_name].extend(
            _int_cmps(
                predicate=property_name,
                eq=eq,
                gt=gt,
                ge=ge,
                lt=lt,
                le=le,
            )
        )
        return self

    @classmethod
    @abc.abstractmethod
    def node_schema(cls) -> "Schema":
        return cast("Schema", None)

    @classmethod
    def associated_viewable(cls) -> Type[V]:
        return cast("Type[V]", cls.node_schema().associated_viewable())

    def neighbor_filters(self) -> List[Tuple[str, EdgeFilter[Q]]]:
        return [
            (edge_name, edge_filter)
            for edge_name, edge_filter in self._edge_filters.items()
        ]

    def property_filters(self) -> List[Tuple[str, List[List["Cmp"]]]]:
        return [
            (property_name, property_filter)
            for property_name, property_filter in self._property_filters.items()
        ]

    def clear_property_filters(self):
        self._property_filters = defaultdict(list)

    def clear_neighbor_filters(self):
        self._edge_filters = defaultdict(list)

    def set_property_filters(self, property_name: str, filters: List[List["Cmp"]]):
        self._property_filters[property_name].extend(filters)

    def set_neighbor_filters(self, edge_name: str, filters: EdgeFilter[Q]):
        self._edge_filters[edge_name].extend(filters)

    def query(self, graph_client: GraphClient, first: int) -> List[V]:
        var_alloc, query = gen_query(self, "q0", first=first)

        variables = {v: k for k, v in var_alloc.allocated.items()}
        txn = graph_client.txn(read_only=True)

        with graph_client.txn_context(read_only=True) as txn:
            try:
                qres = json.loads(txn.query(query, variables=variables).json)
            except Exception as e:
                raise QueryFailedException(query, variables) from e

        d = qres.get("q0")
        if d:
            return [
                self.associated_viewable().from_dict(node, graph_client) for node in d
            ]
        return []

    def query_first(
        self,
        graph_client: GraphClient,
        contains_node_key: Optional[str] = None,
        best_effort=False,
    ) -> Optional[V]:
        if contains_node_key:
            var_alloc, query = gen_query_parameterized(self, "q0", contains_node_key, 0)
        else:
            var_alloc, query = gen_query(self, "q0", first=1)

        variables = {v: k for k, v in var_alloc.allocated.items()}

        with graph_client.txn_context(read_only=True, best_effort=best_effort) as txn:
            try:
                qres = json.loads(txn.query(query, variables=variables).json)
            except Exception as e:
                raise QueryFailedException(query, variables) from e

        d = qres.get("q0")
        if d:
            return self.associated_viewable().from_dict(d[0], graph_client)
        return None

    def get_count(
        self,
        graph_client,
        first: int = 100,
    ) -> int:
        var_alloc, query = gen_query(self, "q0", first=first)

        variables = {v: k for k, v in var_alloc.allocated.items()}
        txn = graph_client.txn(read_only=True)

        try:
            qres = json.loads(txn.query(query, variables=variables).json)
        finally:
            txn.discard()

        return int(qres.get("query", {}).get("c", 0))

    def debug_query(self) -> Dict[str, Any]:
        var_alloc, query = gen_query(self, "q0", first=1)
        variables = {v: k for k, v in var_alloc.allocated.items()}
        return {"query": query, "variables": variables}


from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.comparators import Cmp, Eq
from grapl_analyzerlib.query_gen import gen_query, gen_query_parameterized, VarAllocator
