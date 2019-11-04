import abc
import json
import re
from typing import Dict, TypeVar, Tuple, Type, Callable, Iterable
from typing import Optional, List, Union, Any, Set

from pydgraph import DgraphClient

from grapl_analyzerlib.retry import retry


class Or(object):
    def __init__(self, *values: Union[str, int]):
        self.values = values


class Not(object):
    def __init__(self, value: Union[str, int]) -> None:
        self.value = value


class Cmp(object):
    def to_filter(self) -> str:
        pass


class Eq(Cmp):
    def __init__(self, predicate: str, value: Union[str, int, Not]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, str):
            return 'eq({}, "{}")'.format(self.predicate, self.value)
        if isinstance(self.value, int):
            return "eq({}, {})".format(self.predicate, self.value)
        if isinstance(self.value, Not) and isinstance(self.value.value, str):
            return 'NOT eq({}, "{}")'.format(self.predicate, self.value.value)
        if isinstance(self.value, Not) and isinstance(self.value.value, int):
            return "NOT eq({}, {})".format(self.predicate, self.value.value)


class EndsWith(Cmp):
    def __init__(self, predicate: str, value: str) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, Not):
            value = self.value.value
            escaped_value = re.escape(value)
            return "NOT regexp({}, /.*{}/i)".format(self.predicate, escaped_value)
        else:
            escaped_value = re.escape(self.value)
            return "regexp({}, /.*{}/i)".format(self.predicate, escaped_value)


class Rex(Cmp):
    def __init__(self, predicate: str, value: Union[str, Not]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, Not):
            value = self.value.value
            return f"NOT regexp({self.predicate}, /{value}/)"
        else:
            return f"regexp({self.predicate}, /{self.value}/)"


class Gt(Cmp):
    def __init__(self, predicate: str, value: Union[int, Not]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, Not):
            return f"NOT gt({self.predicate}, {self.value})"
        else:
            return f"gt({self.predicate}, {self.value})"


class Lt(Cmp):
    def __init__(self, predicate: str, value: Union[int, Not]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, Not):
            return f"NOT lt({self.predicate}, {self.value})"
        else:
            return f"lt({self.predicate}, {self.value})"


class Has(Cmp):
    def __init__(self, predicate: str) -> None:
        self.predicate = predicate

    def to_filter(self) -> str:
        return f"has({self.predicate})"


class Contains(Cmp):
    def __init__(self, predicate: str, value: Union[str, Not]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:

        if isinstance(self.value, Not):
            value = re.escape(self.value.value)
            return f"NOT regexp({self.predicate}, /{value}/)"
        else:
            value = re.escape(self.value)
            return f"regexp({self.predicate}, /{value}/)"


class Regexp(Cmp):
    def __init__(self, predicate: str, value: Union[str, Not]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:

        if isinstance(self.value, Not):
            value = self.value.value.replace("/", "\\/")
            return f"NOT regexp({self.predicate}, /{value}/)"
        else:
            value = self.value.replace("/", "\\/")
            return f"regexp({self.predicate}, /{value}/)"


def _generate_filter(comparisons_list: List[List[Cmp]]) -> str:
    and_filters = []

    for comparisons in comparisons_list:
        if len(comparisons) > 1:
            comparisons = [c for c in comparisons if not isinstance(c, Has)]
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


def flatten_nodes(root: Any) -> List[Any]:
    node_list = [root]
    already_visited = set()
    to_visit = [root]

    while True:
        if not to_visit:
            break

        next_node = to_visit.pop()

        if next_node in already_visited:
            continue

        neighbors = next_node.get_neighbors()

        for neighbor in neighbors:
            if isinstance(neighbor, list):
                node_list.extend(neighbor)
                to_visit.extend(neighbor)
            else:
                node_list.append(neighbor)
                to_visit.append(neighbor)

        already_visited.add(next_node)

    # Maintaining order is a convenience
    return list(dict.fromkeys(node_list))


def _str_cmps(
    predicate: str,
    eq: Optional[Union[str, List[str], Not, List[Not]]] = None,
    contains: Optional[Union[str, List[str], Not, List[Not]]] = None,
    ends_with: Optional[Union[str, List[str], Not, List[Not]]] = None,
):
    cmps = []

    inner_eq = eq
    if isinstance(eq, Not):
        inner_eq = eq.value
    if isinstance(inner_eq, str):
        cmps.append([Eq(predicate, eq)])
    elif isinstance(inner_eq, list):
        _eq = [Eq(predicate, e) for e in eq]
        cmps.append(_eq)

    inner_contains = contains
    if isinstance(contains, Not):
        inner_contains = contains.value

    if isinstance(inner_contains, str):
        cmps.append([Contains(predicate, contains)])
    elif isinstance(inner_contains, list):
        _contains = [Contains(predicate, e) for e in contains]
        cmps.append(_contains)

    inner_ends_with = ends_with
    if isinstance(ends_with, Not):
        inner_ends_with = ends_with.value

    if isinstance(inner_ends_with, str):
        cmps.append([EndsWith(predicate, ends_with)])
    elif isinstance(inner_ends_with, list):
        _ends_with = [EndsWith(predicate, e) for e in ends_with]
        cmps.append(_ends_with)

    if not (eq or contains or ends_with):
        cmps.append([Has(predicate)])

    return cmps


def _int_cmps(
    predicate: str,
    eq: Optional[Union[int, List, Not, List[Not]]] = None,
    gt: Optional[Union[int, List, Not, List[Not]]] = None,
    lt: Optional[Union[int, List, Not, List[Not]]] = None,
) -> List[List[Cmp]]:
    cmps = []

    inner_eq = eq
    if isinstance(eq, Not):
        inner_eq = eq.value

    if isinstance(inner_eq, int):
        cmps.append([Eq(predicate, eq)])
    elif isinstance(inner_eq, list):
        _eq = [Eq("last_seen_timestamp", e) for e in eq]
        cmps.append(_eq)

    inner_gt = gt
    if isinstance(gt, Not):
        inner_gt = gt.value

    if isinstance(inner_gt, int):
        cmps.append([Gt(predicate, gt)])
    elif isinstance(inner_gt, list):
        _gt = [Gt("last_seen_timestamp", e) for e in gt]
        cmps.append(_gt)

    inner_lt = lt
    if isinstance(lt, Not):
        inner_lt = lt.value

    if isinstance(inner_lt, int):
        cmps.append([Lt(predicate, lt)])
    elif isinstance(inner_lt, list):
        _lt = [Lt(predicate, e) for e in lt]
        cmps.append(_lt)

    if not (eq or gt or lt):
        cmps.append([Has(predicate)])

    return cmps


PropertyFilter = List[List[Cmp]]
StrCmp = Union[str, List[str], Not, List[Not]]
IntCmp = Union[int, List[int], Not, List[Not]]

EdgeFilter = Optional[Any]

V = TypeVar("V", bound="Viewable")


class Viewable(abc.ABC):
    def __init__(self, dgraph_client: DgraphClient, node_key: str, uid: str):
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid

    @staticmethod
    @abc.abstractmethod
    def get_property_types() -> List[Tuple[str, Callable[[Any], Union[str, int]]]]:
        pass

    @staticmethod
    @abc.abstractmethod
    def get_edge_types() -> List[Tuple[str, Union[List[Type[V]], Type[V]]]]:
        pass

    @abc.abstractmethod
    def get_property_tuples(self) -> List[Tuple[str, Any]]:
        pass

    @abc.abstractmethod
    def get_edge_tuples(self) -> List[Tuple[str, Any]]:
        pass

    def get_property(
        self, prop_name: str, prop_type: Callable[[Any], Union[str, int]]
    ) -> Optional[Union[str, int]]:
        node_key_prop = ""
        if prop_name != "node_key":
            node_key_prop = "node_key"
        query = f"""
            {{
                res(func: uid("{self.uid}")) @cascade {{
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
        if not raw_prop or not raw_prop[0].get(prop_name):
            return None

        prop = prop_type(raw_prop[0][prop_name])

        return prop

    @retry(delay=0.05)
    def get_properties(
        self, prop_name: str, prop_type: Callable[[Any], Union[str, int]]
    ) -> List[str]:
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

        return props

    @retry(delay=0.05)
    def get_edge(self, edge_name: str, edge_type: V) -> Optional[V]:
        query = f"""
            {{
                res(func: uid("{self.uid}")) {{
                    uid,
                    node_key,
                    {edge_name} {{
                        uid,
                        node_type,
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
        if not raw_edge or not raw_edge[0].get(edge_name):
            return None

        edge = edge_type.from_dict(self.dgraph_client, raw_edge[0][edge_name])
        return edge

    @retry(delay=0.05)
    def get_edges(self, edge_name: str, edge_type: Type[V]) -> List[V]:
        query = f"""
            {{
                res(func: uid("{self.uid}")) {{
                    uid,
                    node_key,
                    {edge_name} {{
                        uid,
                        node_type,
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

        raw_edges = raw_edges[0][edge_name]
        edges = [edge_type.from_dict(self.dgraph_client, f) for f in raw_edges]

        return edges

    @classmethod
    def from_dict(cls: Type[V], dgraph_client: DgraphClient, d: Dict[str, Any]) -> V:
        properties = {}
        if d.get("node_type"):
            properties["node_type"] = d["node_type"]

        for prop, into in cls.get_property_types():
            val = d.get(prop)
            if val:
                val = into(val)
                properties[prop] = val

        edges = {}
        for edge_tuple in cls.get_edge_types():
            edge_name, ty = edge_tuple[0], edge_tuple[1]
            raw_edge = d.get(edge_name, None)

            if not raw_edge:
                continue

            if isinstance(ty, List):
                ty = ty[0]

                if d.get(edge_name, None):
                    _edges = [ty.from_dict(dgraph_client, f) for f in d[edge_name]]
                    if len(edge_tuple) == 3:
                        edges[edge_tuple[2]] = _edges
                    else:
                        edges[edge_name] = _edges

            else:
                edge = ty.from_dict(dgraph_client, raw_edge[0])
                if len(edge_tuple) == 3:
                    edges[edge_tuple[2]] = edge
                else:
                    edges[edge_name] = edge

        cleaned_edges = {}
        for edge_name, edge in edges.items():
            if edge_name[0] == "~":
                edge_name = edge_name[1:]
            cleaned_edges[edge_name] = edge

        return cls(
            dgraph_client=dgraph_client,
            node_key=d["node_key"],
            uid=d["uid"],
            **properties,
            **cleaned_edges,
        )


Q = TypeVar("Q", bound="Queryable")


class Queryable(abc.ABC):
    def __init__(self, view_type: Type[V]) -> None:
        self._node_key = Has("node_key")  # type: Cmp
        self._uid = None  # type: Optional[Cmp]
        self.view_type = view_type

    @abc.abstractmethod
    def get_unique_predicate(self) -> Optional[str]:
        pass

    @abc.abstractmethod
    def get_node_type_name(self) -> Optional[str]:
        pass

    @abc.abstractmethod
    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        pass

    @abc.abstractmethod
    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        pass

    @abc.abstractmethod
    def get_reverse_edges(self) -> List[Tuple[str, Any, str]]:
        pass

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        if not self._uid:
            return []
        return [[self._uid]]

    def with_node_key(self: Q, node_key: str) -> Q:
        self._node_key = Eq("node_key", node_key)
        return self

    def with_uid(self: Q, eq: Union[str, Not]) -> Q:
        self._uid = Eq("uid", eq)
        return self

    def get_property_names(self) -> List[str]:
        return [p[0] for p in self.get_properties() if p and p[1]]

    def get_edges(self) -> List[Tuple[str, Any]]:
        all_edges = []
        all_edges.extend(self.get_forward_edges())
        all_edges.extend(self.get_reverse_edges())
        return [e for e in all_edges if e and e[1]]

    def get_neighbors(self) -> List[Q]:
        return [e[1] for e in self.get_edges() if e and e[1]]

    @retry(delay=0.05)
    def query(
        self,
        dgraph_client: DgraphClient,
        contains_node_key: Optional[str] = None,
        first: Optional[int] = 1000,
    ) -> List[V]:
        if contains_node_key:
            first = 1
        query_str = generate_query(
            query_name="res",
            binding_modifier="res",
            root=self,
            contains_node_key=contains_node_key,
            first=first
        )

        txn = dgraph_client.txn(read_only=True)
        try:
            raw_views = json.loads(txn.query(query_str).json)['res']
        finally:
            txn.discard()

        if not raw_views:
            return []

        return [
            self.view_type.from_dict(dgraph_client, raw_view) for raw_view in raw_views
        ]

    def query_first(
        self, dgraph_client: DgraphClient, contains_node_key: Optional[str] = None
    ) -> Optional[V]:
        res = self.query(dgraph_client, contains_node_key, first=1)
        if res:
            return res[0]
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
            count=True
        )


        txn = dgraph_client.txn(read_only=True)
        try:
            raw_count = json.loads(txn.query(query_str).json)['res']
        finally:
            txn.discard()

        if not raw_count:
            return 0
        else:
            return raw_count[0].get("count", 0)

    def to_query(self, count: bool = False, first: Optional[int] = None) -> str:
        return generate_query(
                query_name="res",
                binding_modifier="res",
                root=self,
                first=first,
                count=count
            )

    def _filters(self) -> str:
        inner_filters = []

        for prop in self.get_properties():
            if not prop[1]:
                continue
            f = _generate_filter(prop[1])
            if f:
                inner_filters.append(f)

        if not inner_filters:
            return ""

        return f"@filter({'AND'.join(inner_filters)})"


def get_single_equals_predicate(query: Queryable) -> Optional[Cmp]:
    for prop in query.get_properties():
        prop_name, prop = prop
        # prop is missing or has OR logic
        if not prop or len(prop) != 1:
            continue

        if len(prop[0]) != 1:
            continue

        if isinstance(prop[0][0], Eq):
            return prop[0][0]

    return None


def func_filter(query: Queryable) -> str:
    type_name = query.get_node_type_name()
    single_predicate = get_single_equals_predicate(query)
    if query._node_key and isinstance(query._node_key, Eq):
        return query._node_key.to_filter()
    elif type_name:
        return f'eq(node_type, "{type_name}")'
    elif single_predicate:
        return single_predicate.to_filter()
    elif query.get_unique_predicate():
        return f"has({query.get_unique_predicate()})"
    else:
        # worst case, we have to search every node :(
        return "has(node_key)"


def check_edge(query, edge_name, neighbor, visited):
    if edge_name[0] == '~':
        if (neighbor, edge_name[1:], query) in visited:
            return True
        else:
            visited.add((neighbor, edge_name[1:], query))

    else:
        if (query, edge_name, neighbor) in visited:
            return True
        else:
            visited.add((query, edge_name, neighbor))
    return False


def generate_var_block(
        query: Queryable,
        root: Queryable,
        root_binding: str,
        visited=None,
        should_filter=False
) -> str:
    """
    Generate a var block for this node's query, including all nested neighbors
    """
    if not visited:
        visited = set()

    if query in visited:
        return ""

    visited.add(query)

    all_blocks = []
    for edge in query.get_edges():
        edge_name, neighbor = edge[0], edge[1]

        if check_edge(query, edge_name, neighbor, visited):
            continue

        neighbor_block = generate_var_block(neighbor, root, root_binding, visited, should_filter)

        neighbor_prop = neighbor.get_unique_predicate()
        prop_names = ", ".join(
            [q for q in neighbor.get_property_names()
             if q not in ('node_key', neighbor.get_unique_predicate())]
        )

        formatted_binding = ""
        if neighbor == root and root_binding:
            formatted_binding = root_binding + " as "

        filters = ""
        if should_filter:
            filters = neighbor._filters()

        block = f"""
            {formatted_binding}{edge_name} {filters} {{
                uid,
                node_key,
                {prop_names}
                {neighbor_prop}
                {neighbor_block}
            }}
        """
        all_blocks.append(block)

    return "\n".join(all_blocks)


def generate_root_var(query: Queryable, root: Queryable, root_binding: str, node_key=None) -> str:
    blocks = generate_var_block(query, root, root_binding)

    formatted_binding = ""
    if query == root and root_binding:
        formatted_binding = root_binding + " as "

    if node_key:
        func = f'eq(node_key, "{node_key}"), first: 1'
    else:
        func = func_filter(query)

    prop_names = ", ".join([q for q in query.get_property_names() if q not in ('node_key', query.get_unique_predicate())])
    var_block = f"""
        {formatted_binding}var(func: {func}) @cascade {{
            uid,
            node_key,
            {prop_names}
            {query.get_unique_predicate()}
            {blocks}
        }}
    """

    return var_block



def generate_root_vars(
        query: Queryable,
        binding_modifier: str,
        contains_node_key=None,
) -> Tuple[str, List[str]]:
    """
        Generates root var blocks, and returns bindings associated with the blocks
    """
    var_blocks = []
    root_bindings = []
    for i, node_query in enumerate(traverse_query_iter(query)):
        root_binding = f"RootBinding{binding_modifier}{i}"
        var_block = generate_root_var(node_query, query, root_binding, contains_node_key)
        var_blocks.append(var_block)
        root_bindings.append(root_binding)

    return "\n".join(var_blocks), root_bindings


def generate_coalescing_query(
        query_name: str,
        root: Queryable,
        root_bindings: List[str],
) -> str:
    cs_bindings = ', '.join(root_bindings)

    filtered_var_blocks = generate_var_block(
        root,
        root,
        "",
        should_filter=True,
    )
    root_filters = root._filters()

    prop_names = ", ".join([q for q in root.get_property_names() if q not in ('node_key', root.get_unique_predicate())])

    return f"""
            {query_name}Coalesce as var(func: uid({cs_bindings}))
            @cascade
    
            {root_filters}
    
            {{
                uid,
                {prop_names},
                node_key,
                {root.get_unique_predicate()},
                {filtered_var_blocks}
            }}
          """


def generate_inner_query(
        query_name: str,
        root: Queryable,
        root_bindings: List[str],
        first=None,
        count=False
) -> str:
    filtered_var_blocks = generate_var_block(
        root,
        root,
        "",
        should_filter=True,
    )
    root_filters = root._filters()

    fmt_first = ""
    if first:
        fmt_first = f", first: {first}"

    fmt_count = ""
    if count:
        fmt_count = "count(uid)"

    prop_names = ", ".join([q for q in root.get_property_names() if q not in ('node_key', root.get_unique_predicate())])

    coalesce_var = generate_coalescing_query(
        query_name,
        root,
        root_bindings,
    )

    return f"""
        {coalesce_var}
    
        {query_name}(func: uid({query_name}Coalesce) {fmt_first})
        @cascade

        {root_filters}

        {{
            uid,
            {fmt_count},
            {prop_names},
            node_key,
            {root.get_unique_predicate()},
            {filtered_var_blocks}
        }}
      """

def generate_query(
        query_name: str,
        root: Queryable,
        binding_modifier: str,
        contains_node_key=None,
        first=None,
        count=False,
) -> str:

    var_blocks, root_bindings = generate_root_vars(
        root,
        binding_modifier,
        contains_node_key
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

def _traverse_query_iter(node: Queryable, visited: Set[Queryable]) -> Iterable[Queryable]:
    if node in visited:
        return

    visited.add(node)

    yield node
    for neighbor in node.get_neighbors():
        for t in _traverse_query_iter(neighbor, visited):
            yield t


def traverse_query_iter(node: Queryable) -> Iterable[Queryable]:
    for t in _traverse_query_iter(node, visited=set()):
        yield t


def _traverse_query(node: Queryable, f: Callable[[Queryable], None], visited: Set[Queryable]):
    if node in visited:
        return

    visited.add(node)

    f(node)
    for neighbor in node.get_neighbors():
        _traverse_query(neighbor, f, visited)


def traverse_query(node: Queryable, f: Callable[[Queryable], None]):
    f(node)
    visited = set()
    visited.add(node)
    for neighbor in node.get_neighbors():
        _traverse_query(neighbor, f, visited)

