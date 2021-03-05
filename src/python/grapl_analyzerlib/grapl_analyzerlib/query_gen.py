from copy import deepcopy

from typing import Dict, Set, Optional, Iterator, Tuple, List, NewType

# Represents a string like "$a" or "$b"
VarPlaceholder = NewType("VarPlaceholder", str)


def _placeholder_generator(prefix: str) -> Iterator[VarPlaceholder]:
    curr = VarPlaceholder("$" + (prefix or "") + "a")

    while True:
        yield curr
        last_char = curr[-1]
        if last_char == "z":
            # append a new digits place
            curr = VarPlaceholder(curr + "a")
        else:
            # mutate last digit by 1
            curr = VarPlaceholder(curr[:-1] + chr(ord(last_char) + 1))


class VarAllocator(object):
    def __init__(self, prefix: Optional[str] = None):
        self.prefix = prefix
        self.placeholder_generator: Iterator[VarPlaceholder] = _placeholder_generator(
            self.prefix
        )
        # Value to var identifier, i.e. "hello" => "$a"
        self.allocated: Dict[str, VarPlaceholder] = dict()
        self.typemap: Dict[VarPlaceholder, str] = dict()

    def alloc(self, cmp: "Cmp") -> VarPlaceholder:
        if isinstance(cmp, Has):
            raise NotImplementedError
            return ""
        else:
            cmp_value = str(extract_value(cmp.value))

            var: Optional[VarPlaceholder] = self.allocated.get(cmp_value, None)
            if not var:
                var = next(self.placeholder_generator)
                self.allocated[cmp_value] = var
            self.typemap[var] = dgraph_prop_type(cmp)

            return var

    def reset(self) -> None:
        self.placeholder_generator = _placeholder_generator(self.prefix)
        self.allocated.clear()


def is_regex_based(cmp: "Cmp") -> bool:
    return (
        isinstance(cmp, Contains)
        or isinstance(cmp, StartsWith)
        or isinstance(cmp, EndsWith)
        or isinstance(cmp, Rex)
    )


def gen_prop_filters(q: "Queryable", var_alloc: VarAllocator) -> Optional[str]:
    prop_filter_str = ""
    prop_filters = []
    for (prop_name, or_filter) in q.property_filters():
        or_filters = []

        for and_filter in or_filter:
            andeds = []
            for f in and_filter:
                f = deepcopy(f)
                if isinstance(f, Has) or is_regex_based(f):
                    continue  # We don't need a filter for Has, @cascade will sort that out for us
                else:
                    f.value = var_alloc.alloc(f)
                    serialized = f.to_filter()
                andeds.append(serialized)
            if not andeds:
                continue

            if len(andeds) == 1:
                anded = " AND ".join(andeds)
            else:
                anded = "(" + " AND ".join(andeds) + ")"
            or_filters.append(anded)
        if not or_filters:
            continue
        if len(or_filters) == 1:
            orded = " OR ".join(or_filters)
        else:
            orded = "(" + " OR ".join(or_filters) + ")"
        prop_filters.append(orded)

    if not prop_filters:
        return None

    prop_filter_str += " AND ".join(prop_filters)

    return prop_filter_str


def find_func(q: "Queryable", var_alloc: VarAllocator) -> str:
    """
    `find_func` will look for the most optimal filter.

    * Singular EQ on a unique value
    * Singular eq on a non-unique value
    * <todo, more optimized funcs>
    * Node type

    :param q:
    :return:
    """
    and_filter: List[Cmp]

    singular_eq_nu = None

    for (prop_name, or_filter) in q.property_filters():
        for and_filter in or_filter:
            if len(and_filter) == 1 and (
                (isinstance(and_filter[0], Eq) or isinstance(and_filter[0], IntEq))
                and not and_filter[0].negated
            ):
                filter = deepcopy(and_filter[0])
                if prop_name == "node_key":
                    filter.value = var_alloc.alloc(Eq("node_key", filter.value))
                    return filter.to_filter()
                elif prop_name == "uid":
                    filter.value = var_alloc.alloc(Eq("uid", filter.value))
                    return filter.to_filter()
                else:
                    filter.value = var_alloc.alloc(Eq(filter.predicate, filter.value))
                    singular_eq_nu = filter.to_filter()

    if singular_eq_nu:
        return singular_eq_nu

    return f"type({q.node_schema().self_type()})"


def zip_graph(q: "Queryable", into: "Queryable", visited=None):
    if not visited:
        visited = set()

    if q._id in visited:
        return

    visited.add(q._id)

    for prop_name, prop_filter in q.property_filters():
        into.set_property_filters(prop_name, prop_filter)

    # Create a neighbor that represents the combined structure of all neighbors for a given edge name
    for edge_name, neighbor_filters in q.neighbor_filters():
        if not neighbor_filters:
            continue
        _merged_neighbor = neighbor_filters[0]
        if isinstance(_merged_neighbor, tuple) or isinstance(_merged_neighbor, list):
            merged_neighbor = type(_merged_neighbor[0])()
        else:
            merged_neighbor = type(_merged_neighbor)()

        for neighbor_filter in neighbor_filters:
            if isinstance(neighbor_filter, tuple) or isinstance(neighbor_filter, list):
                if not neighbor_filter:
                    continue
                for inner_neighbor_filter in neighbor_filter:
                    for (
                        prop_name,
                        prop_filter,
                    ) in inner_neighbor_filter.property_filters():
                        merged_neighbor.set_property_filters(prop_name, prop_filter)
                    # zip_graph(inner_neighbor_filter, merged_neighbor, visited)

            else:
                for prop_name, prop_filter in neighbor_filter.property_filters():
                    merged_neighbor.set_property_filters(prop_name, prop_filter)
                # zip_graph(neighbor_filter, merged_neighbor, visited)

        merged = type(q)()
        zip_graph(merged_neighbor, merged, visited)
        into.set_neighbor_filters(edge_name, [merged])


def into_query_block(
    q: "Queryable",
    var_alloc: VarAllocator,
    visited=None,
    depth=None,
    should_filter=True,
    should_alias=True,
    binding: Optional[str] = None,
    root_node: Optional["Queryable"] = None,
) -> Tuple[str, str]:
    """
    Returns the property block and the filters
    :param q:
    :param var_alloc:
    :return:
    """
    if not visited:
        visited = set()

    if q._id in visited:
        return ("", "")

    visited.add(q._id)

    if not depth:
        depth = 0

    depth += 1

    # Generate the filter block
    if should_filter:
        filter = gen_prop_filters(q, var_alloc) or ""
    else:
        filter = ""

    tabs = "\t" * 3 + "\t" * depth
    # Generate the edges (by calling into_query_block on them)

    is_expand = False
    neighbors = ""
    for edge_name, neighbor_filters in q.neighbor_filters():
        for neighbor_filter in neighbor_filters:
            if isinstance(neighbor_filter, tuple) or isinstance(neighbor_filter, list):
                # Generate AND query for neighbor
                for i, inner_neighbor_filter in enumerate(neighbor_filter):
                    (neighbor_filter_str, neighbor_properties) = into_query_block(
                        inner_neighbor_filter,
                        var_alloc,
                        visited,
                        depth,
                        should_filter,
                        should_alias,
                        binding,
                        root_node,
                    )
                    if not neighbor_properties:
                        continue

                    filter_str = ""
                    if should_filter and neighbor_filter_str:
                        filter_str = f"@filter({neighbor_filter_str})"

                    alias = ""
                    if should_alias and len(neighbor_filter) != 1:
                        alias = f"{edge_name}_{depth}_{i} : "

                    if edge_name.startswith("expand(") and edge_name.endswith(")"):
                        is_expand = True

                    formatted_var_name = ""
                    if binding and root_node:
                        if inner_neighbor_filter._id == root_node._id:
                            formatted_var_name = f"{binding} as "

                    neighbors += (
                        f"\n{formatted_var_name}  {alias} {edge_name} {filter_str} {{"
                        + f"{neighbor_properties}"
                        + "}"
                    )

            else:
                # Generate OR logic for query
                neighbor_filter_str, neighbor_properties = into_query_block(
                    neighbor_filter,
                    var_alloc,
                    visited,
                    depth,
                    should_filter,
                    should_alias,
                    binding,
                    root_node,
                )
                if not neighbor_properties:
                    continue

                filter_str = ""
                if should_filter and neighbor_filter_str:
                    filter_str = f"@filter({neighbor_filter_str})"

                formatted_var_name = ""
                if binding and root_node:
                    if neighbor_filter._id == root_node._id:
                        formatted_var_name = f"{binding} as "
                neighbors += (
                    f"\n{formatted_var_name} {edge_name} {filter_str} {{"
                    + f"{neighbor_properties}"
                    + "}"
                )

    # Grab the properties
    if is_expand:
        properties = f"\n{tabs}".join(["uid", "dgraph.type"])
    else:
        always = {"uid", "dgraph.type", "node_key"}

        properties = f"\n{tabs}".join(
            [prop[0] for prop in q.property_filters() if prop[1] or prop[0] in always]
        )

    properties += neighbors

    return (filter, properties)


def into_vars_list(vars_alloc: VarAllocator) -> str:
    return ", ".join([f"{vn}: {vt}" for vn, vt in vars_alloc.typemap.items()])


def into_var_query(
    q: "Queryable",
    var_name: str,
    vars_alloc: VarAllocator,
    func: Optional[str] = None,
    first: Optional[int] = None,
    binding: Optional[str] = None,
    root_node: Optional["Queryable"] = None,
    cascade: Optional[bool] = None,
) -> Tuple[str, str]:
    func = func or find_func(q, vars_alloc)
    filters, block = into_query_block(
        q, vars_alloc, binding=binding, root_node=root_node
    )

    f_first = ""
    if first:
        f_first = f", first: {first}"

    cascade_str = ""
    if cascade:
        cascade_str = "@cascade"
    formatted_var_name = ""
    if var_name:
        formatted_var_name = f"{var_name} as "

    filter_str = ""
    if filters:
        filter_str = f"@filter({filters})"
    query = f"""
            # into_var_query
            {formatted_var_name} var(func: {func} {f_first})
            {filter_str}
            {cascade_str}
            {{
                {block}
            }}
    """

    return query, block


def gen_coalescing_query(
    q: "Queryable", var_alloc: VarAllocator, query_name: str, bindings: List[str]
) -> Tuple[str, str]:

    __merged_filters, merged_query_block = into_query_block(
        q,
        var_alloc,
        should_filter=True,
        should_alias=False,
    )

    cs_bindings = ", ".join(bindings)

    coalescing_query_name = f"{query_name}Coalesce"

    coalescing_query = f"""
        {coalescing_query_name} as var(func: uid({cs_bindings}))
        @cascade
        {{
            {merged_query_block}
        }}
      """

    return coalescing_query_name, coalescing_query


def gen_query_parameterized(
    q: "Queryable",
    query_name: str,
    contains_node_key: str,
    depth: int,
    binding_modifier: Optional[str] = None,
    vars_alloc: Optional[VarAllocator] = None,
) -> Tuple[VarAllocator, str]:
    binding_modifier = binding_modifier or ""
    vars_alloc = vars_alloc or VarAllocator()

    bindings = []
    var_queries = []

    node_key_var = vars_alloc.alloc(Eq("node_key", contains_node_key))
    for i, node in enumerate(traverse_query_iter(q)):
        func = f"eq(node_key, {node_key_var}), first: 1"
        binding = f"{binding_modifier}Binding{depth}_{i}"
        bindings.append(binding)
        var_name = ""

        if node._id == q._id:
            var_name = binding

        var_query, var_block = into_var_query(
            node,
            var_name,
            vars_alloc,
            func=func,
            binding=binding,
            root_node=q,
        )

        var_queries.append(var_query)

    formatted_var_queries = "\n".join(var_queries)

    merged = type(q)()
    zip_graph(q, merged)
    __merged_filters, merged_query_block = into_query_block(
        merged,
        VarAllocator(),
        should_filter=False,
        should_alias=False,
    )

    vars_list = into_vars_list(vars_alloc)

    coalescing_query_name, coalescing_query = gen_coalescing_query(
        merged, vars_alloc, query_name, bindings
    )

    query = f"""
        query {query_name}({vars_list}) {{
            {formatted_var_queries}

            {coalescing_query}

            {query_name}(func: uid({coalescing_query_name}), first: 1) @cascade {{
                {merged_query_block}
            }}
        }}
    """

    return vars_alloc, query


def gen_query(
    q: "Queryable",
    query_name: str,
    first: Optional[int] = None,
    count=False,
) -> Tuple[VarAllocator, str]:
    if not first:
        first = 1

    vars_alloc = VarAllocator()

    func = find_func(q, vars_alloc)
    var_query, var_block = into_var_query(q, "q0", vars_alloc, func=func, cascade=True)

    if count:
        merged_query_block = f"c as count(uid)"
    else:
        merged = type(q)()
        zip_graph(q, merged)
        __merged_filters, merged_query_block = into_query_block(
            merged,
            VarAllocator(),
            should_filter=False,
            should_alias=False,
        )

    vars_list = into_vars_list(vars_alloc)

    f_first = ""
    if first:
        f_first = f", first: {first}"
    query = f"""
        query {query_name}({vars_list}) {{
            {var_query}

            {query_name}(func: uid(q0) {f_first}) @cascade {{
                {merged_query_block}
            }}
        }}
    """

    return vars_alloc, query


def traverse_query_iter(
    root_q: "Queryable", visited: Optional[Set["Queryable"]] = None
) -> Iterator["Queryable"]:
    if visited is None:
        visited = set()

    if root_q in visited:
        return
    yield root_q

    for edge_name, neighbor_filters in root_q.neighbor_filters():
        if not neighbor_filters:
            continue

        visited.add(root_q)

        for neighbor_filter in neighbor_filters:
            if isinstance(neighbor_filter, tuple) or isinstance(neighbor_filter, list):
                for n_filter in neighbor_filter:
                    for nested in traverse_query_iter(n_filter, visited):
                        yield nested
            else:
                for nested in traverse_query_iter(neighbor_filter, visited):
                    yield nested


def traverse_query_neighbors_iter(
    root_q: "Queryable", visited: Optional[Set["Queryable"]] = None
) -> Iterator[Tuple["Queryable", str, "EdgeFilter"]]:
    if visited is None:
        visited = set()

    if root_q in visited:
        return

    for edge_name, neighbor_filters in root_q.neighbor_filters():
        if not neighbor_filters:
            continue
        if root_q not in visited:
            yield root_q, edge_name, neighbor_filters

        visited.add(root_q)

        for neighbor_filter in neighbor_filters:
            if isinstance(neighbor_filter, tuple) or isinstance(neighbor_filter, list):
                for n_filter in neighbor_filter:
                    for nested in traverse_query_neighbors_iter(n_filter, visited):
                        yield nested
            else:
                for nested in traverse_query_neighbors_iter(neighbor_filter, visited):
                    yield nested


from grapl_analyzerlib.queryable import Queryable, EdgeFilter
from grapl_analyzerlib.comparators import (
    Cmp,
    Eq,
    IntEq,
    Has,
    extract_value,
    dgraph_prop_type,
    Contains,
    StartsWith,
    EndsWith,
    Rex,
)
