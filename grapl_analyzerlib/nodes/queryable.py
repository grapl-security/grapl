import abc
import json
import uuid
from collections import defaultdict
from typing import (
    TypeVar,
    Type,
    Optional,
    Tuple,
    List,
    Mapping,
    Generic,
    Dict,
    Set,
    Iterable,
    Union,
    cast,
    Any,
)

from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.retry import retry

U = TypeVar("U", bound=Union[str, int])
NQ = TypeVar("NQ", bound="Queryable")
NV = TypeVar("NV", bound="Viewable")


class Queryable(abc.ABC, Generic[NV]):
    def __init__(self, view_type: Type["NV"]) -> None:
        self._node_key = [[Has("node_key")]]  # type: List[List[Cmp[str]]]
        self._uid = None  # type: Optional[Cmp[str]]
        self._query_id = str(uuid.uuid4())

        self.view_type = view_type  # type: Type[NV]

        self.dynamic_forward_edge_filters = {}  # type: Dict[str, 'Queryable']
        self.dynamic_reverse_edge_filters = (
            {}
        )  # type: Dict[str, Tuple['Queryable', str]]
        self.dynamic_property_filters = defaultdict(
            list
        )  # type: Dict[str, 'PropertyFilter[Property]']

    def extend(self, extended_type: Type[NQ]) -> NQ:
        return extended_type(self.view_type)

    def with_node_key(self: 'NQ', eq: str) -> 'NQ':
        self._node_key = [[Eq("node_key", eq)]]
        return self

    @abc.abstractmethod
    def _get_unique_predicate(self) -> Optional[Tuple[str, "PropertyT"]]:
        """If the Node has a guaranteed unique predicate, return its name and type"""

    @abc.abstractmethod
    def _get_node_type_name(self) -> str:
        """Every query must define the node type"""

    @abc.abstractmethod
    def _get_property_filters(self) -> Mapping[str, "PropertyFilter[Property]"]:
        """Defines the filters for every property in the query"""

    @abc.abstractmethod
    def _get_forward_edges(self) -> Mapping[str, "Queryable"]:
        """:returns The built up comparisons for all forward edges"""

    @abc.abstractmethod
    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        """:returns The built up comparisons for all reverse edges"""

    def _get_unique_predicate_name(self) -> Optional[str]:
        unique_pred = self._get_unique_predicate()
        if unique_pred is None:
            return None

        return unique_pred[0]

    def get_forward_edges(self) -> Mapping[str, "Queryable"]:
        forward_edges = self._get_forward_edges()
        return {**forward_edges, **self.dynamic_forward_edge_filters}

    def get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        reverse_edges = self._get_reverse_edges()
        return {**reverse_edges, **self.dynamic_reverse_edge_filters}

    def get_edge_filters(
        self
    ) -> Mapping[str, Union["Queryable", Tuple["Queryable", str]]]:
        return {**self.get_forward_edges(), **self.get_reverse_edges()}

    def get_property_filters(self) -> Mapping[str, "PropertyFilter[Property]"]:
        prop_filters = self._get_property_filters()
        if not prop_filters.get('node_key'):
            prop_filters['node_key'] = self._node_key

        return {**prop_filters, **self.dynamic_property_filters}

    def get_property_names(self) -> List[str]:
        prop_names = [
            p[0] for p in self.get_property_filters().items() if p[1] is not None
        ]
        prop_names.append("node_key")
        return list(set(prop_names))

    def set_forward_edge_filter(self, edge_name: str, edge_filter: "Queryable") -> None:
        self.dynamic_forward_edge_filters[edge_name] = edge_filter

    def set_reverse_edge_filter(
        self, edge_name: str, edge_filter: "Queryable", forward_name: str
    ) -> None:
        self.dynamic_reverse_edge_filters[edge_name] = edge_filter, forward_name

    def set_str_property_filter(
        self, property_name: str, property_filter: "List[List[Cmp[str]]]"
    ) -> None:
        self.dynamic_property_filters[property_name].extend(cast(Any, property_filter))

    def set_int_property_filter(
        self, property_name: str, property_filter: "List[List[Cmp[int]]]"
    ) -> None:
        self.dynamic_property_filters[property_name].extend(cast(Any, property_filter))

    def query(
        self,
        dgraph_client: DgraphClient,
        contains_node_key: Optional[str] = None,
        first: Optional[int] = 1000,
    ) -> List["NV"]:
        if not first:
            first = 1000

        if contains_node_key:
            first = 1

        query_str = generate_query(
            query_name="res",
            binding_modifier="res",
            root=self,
            contains_node_key=contains_node_key,
            first=first,
        )

        txn = dgraph_client.txn(read_only=True)
        try:
            raw_views = json.loads(txn.query(query_str).json)["res"]
        except Exception as e:
            raise Exception(query_str, e)
        finally:
            txn.discard()

        if not raw_views:
            return []

        return [
            self.view_type.from_dict(dgraph_client, raw_view) for raw_view in raw_views
        ]

    def query_first(
        self, dgraph_client: DgraphClient, contains_node_key: Optional[str] = None
    ) -> Optional["NV"]:
        res = self.query(dgraph_client, contains_node_key, first=1)
        if res and isinstance(res, list):
            return cast("NV", res[0])
        if res:
            return cast("NV", res)
        else:
            return None

    @retry(delay=0.05)
    def get_count(
        self,
        dgraph_client: DgraphClient,
        first: Optional[int] = None,
        contains_node_key: Optional[str] = None,
    ) -> int:
        query_str = generate_query(
            query_name="res",
            binding_modifier="res",
            root=self,
            contains_node_key=contains_node_key,
            first=first,
            count=True,
        )

        txn = dgraph_client.txn(read_only=True)
        try:
            raw_count = json.loads(txn.query(query_str).json)["res"]
        finally:
            txn.discard()

        if not raw_count:
            return 0
        else:
            if isinstance(raw_count, list):
                return int(raw_count[0].get("count", 0))
            if isinstance(raw_count, dict):
                return int(raw_count.get("count", 0))
            raise TypeError("raw_count is not list or dict")


def traverse_query_iter(
    node: Queryable, visited: Optional[Set[Queryable]] = None
) -> Iterable[Union["Queryable", Tuple["Queryable", str]]]:

    if visited is None:
        visited = set()
    if node in visited:
        return

    visited.add(node)

    yield node
    for _neighbor in node.get_edge_filters().values():
        if not isinstance(_neighbor, Queryable):
            neighbor = _neighbor[0]
        else:
            neighbor = _neighbor
        for t in traverse_query_iter(neighbor, visited):
            yield t


def check_edge(
    query: Queryable,
    edge_name: str,
    neighbor: Queryable,
    visited: Set[Union[Queryable, Tuple[Queryable, str, Queryable]]],
) -> bool:
    already_seen = False
    if edge_name[0] == "~":
        already_seen = already_seen or ((query, edge_name, neighbor) in visited)
        already_seen = already_seen or ((neighbor, edge_name[1:], query) in visited)

        # Store that we have visited the reverse edge as well as the edge name
        visited.add((query, edge_name, neighbor))
        visited.add((neighbor, edge_name[1:], query))
        visited.add((query, edge_name[1:], neighbor))
        visited.add((neighbor, edge_name, query))
    else:
        already_seen = already_seen or ((query, edge_name, neighbor) in visited)
        already_seen = already_seen or ((neighbor, "~" + edge_name, query) in visited)

        visited.add((query, edge_name, neighbor))
        visited.add((neighbor, "~" + edge_name, query))
        visited.add((query, "~" + edge_name, neighbor))
        visited.add((neighbor, edge_name, query))

    return already_seen


def _generate_filter(comparisons_list: "PropertyFilter[Property]") -> str:
    and_filters = []

    for comparisons in comparisons_list:
        if len(comparisons) > 1:
            comparisons = [c for c in comparisons if not isinstance(c, Has)]
        filters = []
        filters = [comparison.to_filter() for comparison in comparisons]
        filters = [f for f in filters if f]
        if not filters:
            continue
        and_filter = "(" + " AND ".join(filters) + ")"
        and_filters.append(and_filter)

    or_filters = " OR\n\t".join(and_filters)
    if not or_filters:
        return ""
    return "(\n\t{}\n)".format(or_filters)


def _format_filters(property_filters: List["PropertyFilter[Property]"]) -> str:
    inner_filters = []

    for prop in property_filters:

        f = _generate_filter(prop)
        if f:
            inner_filters.append(f)

    if not inner_filters:
        return ""

    return f"@filter({'AND'.join(inner_filters)})"


def _get_single_equals_predicate(query: Queryable) -> Optional["Cmp[Property]"]:
    for prop_name, prop_filter in query.get_property_filters().items():

        # prop is missing or has OR logic
        if len(prop_filter) != 1:
            continue

        if len(prop_filter[0]) != 1:
            continue

        if isinstance(prop_filter[0][0], Eq):
            return prop_filter[0][0]

    return None


def _func_filter(query: Queryable) -> str:
    type_name = query._get_node_type_name()
    single_predicate = _get_single_equals_predicate(query)

    if query._node_key and isinstance(query._node_key, Eq):
        return query._node_key.to_filter()
    elif type_name:
        return f'eq(dgraph.type, "{type_name}")'
    elif single_predicate:
        return single_predicate.to_filter()
    elif query._get_unique_predicate_name():
        return f"has({query._get_unique_predicate_name()})"
    else:
        # worst case, we have to search every node :(
        return "has(node_key)"


def generate_var_block(
    query: Queryable,
    root: Queryable,
    root_binding: str,
    visited: Optional[Set[Union[Queryable, Tuple[Queryable, str, Queryable]]]] = None,
    should_filter: bool = False,
) -> str:
    """
    Generate a var block for this node's query, including all nested neighbors
    """
    if visited is None:
        visited = set()

    if query in visited:
        return ""

    visited.add(query)

    all_blocks = []
    for edge_name, _neighbor in query.get_edge_filters().items():
        forward_name = None
        if not isinstance(_neighbor, Queryable):
            neighbor = _neighbor[0]
            forward_name = _neighbor[1]
        else:
            neighbor = _neighbor

        if check_edge(query, edge_name, neighbor, visited):
            continue

        # if forward_name:
        #     if check_edge(query, forward_name, neighbor, visited):
        #         continue

        neighbor_block = generate_var_block(
            neighbor, root, root_binding, visited, should_filter
        )

        neighbor_prop = neighbor._get_unique_predicate_name() or ""
        prop_names = ", ".join(
            [
                q
                for q in neighbor.get_property_names()
                if q
                not in (
                    "node_key",
                    "dgraph.type",
                    neighbor._get_unique_predicate_name(),
                )
                and q
            ]
        )

        formatted_binding = ""
        if neighbor == root and root_binding:
            formatted_binding = root_binding + " as "

        filters = ""
        if should_filter:
            prop_filters = [pf for pf in neighbor.get_property_filters().values()]
            filters = _format_filters(prop_filters)

        block = f"""
            {formatted_binding}{edge_name} {filters} {{
                uid,
                node_key,
                node_type: dgraph.type,
                {prop_names + ","}
                {neighbor_prop}
                {neighbor_block}
            }}
        """
        all_blocks.append(block)

    return "\n".join(all_blocks)


def generate_root_var(
    query: Queryable, root: Queryable, root_binding: str, node_key: Optional[str] = None
) -> str:
    blocks = generate_var_block(query, root, root_binding)

    formatted_binding = ""
    if query == root and root_binding:
        formatted_binding = root_binding + " as "

    if node_key:
        func = f'eq(node_key, "{node_key}"), first: 1'
    else:
        func = _func_filter(query)

    _prop_names = [
        q
        for q in query.get_property_names()
        if q not in ("node_key", "dgraph.type", query._get_unique_predicate_name())
    ]

    prop_names = ", ".join(_prop_names)

    var_block = f"""
        {formatted_binding}var(func: {func}) @cascade {{
            uid,
            node_key,
            node_type: dgraph.type,
            {prop_names}
            {query._get_unique_predicate_name() or ""}
            {blocks}
        }}
    """

    return var_block


def generate_root_vars(
    query: Queryable, binding_modifier: str, contains_node_key: Optional[str] = None
) -> Tuple[str, List[str]]:
    """
        Generates root var blocks, and returns bindings associated with the blocks
    """
    var_blocks = []
    root_bindings = []
    for i, _node_query in enumerate(traverse_query_iter(query)):
        if not isinstance(_node_query, Queryable):
            node_query = _node_query[0]
        else:
            node_query = _node_query

        root_binding = f"RootBinding{binding_modifier}{i}"
        var_block = generate_root_var(
            node_query, query, root_binding, contains_node_key
        )

        var_blocks.append(var_block)
        root_bindings.append(root_binding)

    return "\n".join(var_blocks), root_bindings


def generate_coalescing_query(
    query_name: str, root: Queryable, root_bindings: List[str]
) -> str:
    cs_bindings = ", ".join(root_bindings)

    filtered_var_blocks = generate_var_block(root, root, "", should_filter=True)
    prop_filters = [pf for pf in root.get_property_filters().values()]
    root_filters = _format_filters(prop_filters)

    prop_names = ", ".join(
        [
            q
            for q in root.get_property_names()
            if q not in ("node_key", "dgraph.type", root._get_unique_predicate_name())
        ]
    )

    if prop_names:
        fmt_prop_names = prop_names + ","
    else:
        fmt_prop_names = ""
    return f"""
            {query_name}Coalesce as var(func: uid({cs_bindings}))
            @cascade
    
            {root_filters}
    
            {{
                uid,
                {fmt_prop_names}
                node_key,
                node_type: dgraph.type,
                {root._get_unique_predicate_name() or ""},
                {filtered_var_blocks}
            }}
          """


def generate_inner_query(
    query_name: str,
    root: Queryable,
    root_bindings: List[str],
    first: Optional[int] = None,
    count: bool = False,
) -> str:
    filtered_var_blocks = generate_var_block(root, root, "", should_filter=True)

    prop_filters = [pf for pf in root.get_property_filters().values()]

    root_filters = _format_filters(prop_filters)

    fmt_first = ""
    if first:
        fmt_first = f", first: {first}"

    fmt_count = ""
    if count:
        fmt_count = "count(uid)"

    prop_names = ", ".join(
        [
            q
            for q in root.get_property_names()
            if q not in ("node_key", "dgraph.type", root._get_unique_predicate_name())
        ]
    )

    coalesce_var = generate_coalescing_query(query_name, root, root_bindings)

    return f"""
        {coalesce_var}
    
        {query_name}(func: uid({query_name}Coalesce) {fmt_first})
        @cascade

        {root_filters}

        {{
            uid,
            {fmt_count},
            {prop_names},
            node_type: dgraph.type,
            node_key,
            {root._get_unique_predicate_name() or ""},
            {filtered_var_blocks}
        }}
      """


def generate_query(
    query_name: str,
    root: Queryable,
    binding_modifier: str,
    contains_node_key: Optional[str] = None,
    first: Optional[int] = None,
    count: bool = False,
) -> str:

    var_blocks, root_bindings = generate_root_vars(
        root, binding_modifier, contains_node_key
    )

    query_header = ""
    if contains_node_key:
        query_header = f"{query_name}"

    if contains_node_key:
        # node_key is a unique property
        first = 1

    return f"""
        query {query_header}
        {{
            {var_blocks}
            {generate_inner_query(query_name, root, root_bindings, first, count)}
        }}
    """


from grapl_analyzerlib.nodes.comparators import Has, Cmp, PropertyFilter, Eq

from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.nodes.viewable import Viewable, NV
