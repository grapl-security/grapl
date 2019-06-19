import re

from typing import Optional, List, Union, Any, Set


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
            escaped_value = re.escape(value)
            return f"NOT regexp({self.predicate}, /{escaped_value}/)"
        else:
            escaped_value = re.escape(self.value)
            return f"regexp({self.predicate}, /{escaped_value}/)"


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
            value = self.value.value
            escaped_value = re.escape(value)
            return 'NOT alloftext({}, "{}")'.format(self.predicate, escaped_value)
        else:
            escaped_value = re.escape(self.value)
            return 'alloftext({}, "{}")'.format(self.predicate, escaped_value)


def get_var_block(
    node: Any, edge_name: str, binding_num: int, root: Any, already_converted: Set[Any]
) -> str:
    var_block = ""
    if node and node not in already_converted:
        var_block = node._get_var_block(binding_num, root, already_converted)
        if node == root:
            var_block = f"Binding{binding_num} as {edge_name} {var_block}"
        else:
            var_block = f"{edge_name} {var_block}"

    return var_block


def _generate_filter(comparisons_list: List[List[Cmp]]) -> str:
    and_filters = []

    for comparisons in comparisons_list:
        filters = [comparison.to_filter() for comparison in comparisons]
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


def _build_expansion(node: Union[Any, Any], already_visited: Set[Any]) -> str:
    if node in already_visited:
        return ""
    already_visited.add(node)

    edges = node.get_edges()

    expanded_edges = []

    for edge, neighbor in edges:
        if neighbor in already_visited:
            continue
        already_visited.add(neighbor)
        neighbor_props = neighbor.get_properties()
        expanded_edge = f"""
                
                    {edge} {{
                        {",".join(neighbor_props)},
                        {_build_expansion(neighbor, already_visited)}
                    }}
                
            """
        expanded_edges.append(expanded_edge)
    if not expanded_edges:
        return ""
    return ",".join([x for x in expanded_edges if x])


def build_expansion(node: Union[Any, Any]) -> str:
    props = node.get_properties()
    edges = node.get_edges()

    expanded_edges = []

    already_visited = {node}

    for edge, neighbor in edges:
        neighbor_props = neighbor.get_properties()
        expanded_edge = f"""
                {edge} {{
                    {",".join([x for x in neighbor_props if x])},
                     
                    {_build_expansion(neighbor, already_visited)}
                
                }}
            """
        expanded_edges.append(expanded_edge)

    return f"""
            {",".join(props)},
            {", ".join([x for x in expanded_edges if x])}
    """


def build_query(
    node: Any,
    var_blocks: List[str],
    bindings: List[str],
    count: bool = False,
    first: Optional[int] = None,
) -> str:

    joined_vars = "\n".join(var_blocks)
    if not count:
        expansion = build_expansion(node)
    else:
        expansion = "count(uid) as c"

    if not first:
        first = ""
    else:
        first = f", first: {first}"

    query = f"""
            {{
                {joined_vars}
            
            res(func: uid({", ".join(bindings)}) {first}) {{
                 {expansion}
            }}
           
           }}
        """
    return query


def get_queries(process_query: Any, node_key: str, count: bool = False):
    all_nodes = flatten_nodes(process_query)
    bindings = []
    var_blocks = []

    for i, node in enumerate(all_nodes):
        bindings.append(f"Binding{i}")
        var_blocks.append(
            node._get_var_block_root(i, root=process_query, node_key=node_key)
        )

    return build_query(process_query, var_blocks, bindings, count)


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


#
# import unittest
# import re
#
#
# class Test(unittest.TestCase):
#     @staticmethod
#     def format_query(query):
#         return re.sub(" +", " ", (query.replace("\t", "").replace("\n", "").strip()))
#
#         # return (
#         #     query
#         #     .replace("\t", "")
#         #     .replace("\n", "")# )
#
#     def test_any_process_key_opt(self):
#         p = ProcessQuery()
#         queries = self.format_query(get_queries(p, node_key="keyA"))
#
#         expected = self.format_query(
#             """
#             {
#                 Binding0 as var(func: eq(node_key, "keyA")) { }
#                 res(func: uid(Binding0), first: 1) {
#                     expand(_all_) {}
#                 }
#             }"""
#         )
#         assert queries == expected, "\n" + queries + "\n" + expected
#
#     def test_has_process_name(self):
#         ProcessQuery().with_process_name()
#         p = ProcessQuery()
#         queries = self.format_query(get_queries(p, node_key="keyA"))
#
#         expected = self.format_query(
#             """
#         {
#             Binding0 as var(func: eq(node_key, "keyA")) { }
#             res(func: uid(Binding0), first: 1) {
#                 expand(_all_) {}
#             }
#         }
#         """
#         )
#         assert queries == expected, "\n" + queries + "\n" + expected
#
#     def test_has_bin_file(self):
#
#         p = ProcessQuery().with_bin_file(FileQuery())
#         queries = self.format_query(get_queries(p, node_key="keyA"))
#
#         expected = self.format_query(
#             """
#         {
#             Binding0 as var(func: eq(node_key, "keyA")) {
#                 bin_file { }
#             }
#
#             var(func: eq(node_key, "keyA")) {
#                 Binding1 as ~bin_file { }
#             }
#             res(func: uid(Binding0, Binding1), first: 1) {
#                 expand(_all_) {expand(_all_) {}}
#             }
#         }
#         """
#         )
#         assert queries == expected, "\n" + queries + "\n" + expected
#
#     def test_has_bin_file_with_path(self):
#
#         p = ProcessQuery().with_bin_file(FileQuery().with_file_path(eq="foo"))
#         queries = self.format_query(get_queries(p, node_key="keyA"))
#
#         expected = self.format_query(
#             """
#         {
#             Binding0 as var(func: eq(node_key, "keyA")) {
#                 bin_file @filter(((eq(file_path, "foo")))) { }
#             }
#             var(func: eq(node_key, "keyA")) @filter(((eq(file_path, "foo"))))  {
#                 Binding1 as ~bin_file { }
#             }
#
#             res(func: uid(Binding0, Binding1), first: 1) {
#                 expand(_all_) {expand(_all_) {}}
#             }
#         }
#         """
#         )
#         assert queries == expected, "\n" + queries + "\n" + expected
#
#
# if __name__ == "__main__":
#     unittest.main()
#     # main()
#
#
