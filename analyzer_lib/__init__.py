from copy import deepcopy
from typing import TypeVar, Iterable, Optional, List, Union, Any

PQ = TypeVar("PQ", bound="ProcessQuery")
FQ = TypeVar("FQ", bound="FileQuery")


def get_var_block(node, edge_name, binding_num, root, already_converted) -> str:
    var_block = ""
    if node and node not in already_converted:
        var_block = node._get_var_block(binding_num, root, already_converted)
        if node == root:
            var_block = f"Binding{binding_num} as {edge_name} {var_block}"
        else:
            var_block = f"{edge_name} {var_block}"

    return var_block


def _generate_filter(comparisons_list) -> str:
    and_filters = []

    for comparisons in comparisons_list:
        filters = [comparison.to_filter() for comparison in comparisons]
        and_filter = "(" + " AND ".join(filters) + ")"
        and_filters.append(and_filter)

    or_filters = " OR\n\t".join(and_filters)
    if not or_filters:
        return ""
    return "(\n\t{}\n)".format(or_filters)


def flatten_nodes(root) -> List[Any]:
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

        node_list.extend(neighbors)

        neighbors.extend(to_visit)
        to_visit = neighbors

        already_visited.add(next_node)

    # Maintaining order is a convenience
    return list(dict.fromkeys(node_list))


def build_query(var_blocks: List[str], bindings: List[str]) -> str:
    joined_vars = "\n".join(var_blocks)
    expansion = ""

    for _i in range(0, len(bindings)):
        expansion += """expand(_all_) {"""

    for _i in range(0, len(bindings)):
        expansion += """}"""

    query = f"""
            {{
                {joined_vars}
            
            res(func: uid({", ".join(bindings)}), first: 1) {{
                 {expansion}
            }}
           
           }}
        """
    return query


def get_queries(process_query, node_key):
    all_nodes = flatten_nodes(process_query)
    bindings = []
    var_blocks = []

    for i, node in enumerate(all_nodes):
        bindings.append(f"Binding{i}")
        var_blocks.append(
            node._get_var_block_root(i, root=process_query, node_key=node_key)
        )

    return build_query(var_blocks, bindings)


class Cmp(object):
    def to_filter(self) -> str:
        pass


class Eq(Cmp):
    def __init__(self, predicate: str, value: Union[str, int]):
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, int):
            return "eq({}, {})".format(self.predicate, self.value)
        return 'eq({}, "{}")'.format(self.predicate, self.value)


class EndsWith(Cmp):
    def __init__(self, predicate: str, value: str):
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        escaped_value = re.escape(self.value)
        return "regexp({}, /.*{}/i)".format(self.predicate, escaped_value)


class Rex(Cmp):
    def __init__(self, predicate: str, value: str):
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        escaped_value = re.escape(self.value)
        return f"regexp({self.predicate}, /{escaped_value}/)"


class Gt(Cmp):
    def __init__(self, predicate: str, value: int):
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        return f"gt({self.predicate}, {self.value}"


class Lt(Cmp):
    def __init__(self, predicate: str, value: int):
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        return f"gt({self.predicate}, {self.value})"


class Has(Cmp):
    def __init__(self, predicate):
        self.predicate = predicate

    def to_filter(self) -> str:
        return f"has({self.predicate})"


class Contains(Cmp):
    def __init__(self, predicate: str, value: str):
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        escaped_value = re.escape(self.value)
        return "regexp({}, /.*{}.*/i)".format(self.predicate, escaped_value)


def _str_cmps(
    predicate: str,
    eq: Optional[Union[str, List]] = None,
    contains: Optional[Union[str, List]] = None,
    ends_with: Optional[Union[str, List]] = None,
):
    cmps = []
    if isinstance(eq, str):
        cmps.append([Eq(predicate, eq)])
    elif isinstance(eq, list):
        _eq = [Eq(predicate, e) for e in eq]
        cmps.append(_eq)

    if isinstance(contains, str):
        cmps.append([Contains(predicate, contains)])
    elif isinstance(contains, list):
        _contains = [Contains(predicate, e) for e in contains]
        cmps.append(_contains)

    if isinstance(ends_with, str):
        cmps.append([EndsWith(predicate, ends_with)])
    elif isinstance(ends_with, list):
        _ends_with = [EndsWith(predicate, e) for e in ends_with]
        cmps.append(_ends_with)

    if not (eq or contains or ends_with):
        cmps.append([Has(predicate)])

    return cmps


def _int_cmps(
    predicate: str,
    eq: Optional[Union[int, List]] = None,
    gt: Optional[Union[int, List]] = None,
    lt: Optional[Union[int, List]] = None,
) -> List[List[Cmp]]:
    cmps = []
    if isinstance(eq, int):
        cmps.append([Eq(predicate, eq)])
    elif isinstance(eq, list):
        _eq = [Eq("last_seen_timestamp", e) for e in eq]
        cmps.append(_eq)

    if isinstance(gt, int):
        cmps.append([Gt(predicate, gt)])
    elif isinstance(gt, list):
        _gt = [Rex("last_seen_timestamp", e) for e in gt]
        cmps.append(_gt)

    if isinstance(lt, int):
        cmps.append([EndsWith(predicate, lt)])
    elif isinstance(lt, list):
        _lt = [Lt(predicate, e) for e in lt]
        cmps.append(_lt)

    if not (eq or gt or lt):
        cmps.append([Has(predicate)])

    return cmps


class ProcessQuery(object):
    def __init__(self):
        # Properties
        self._process_name = []  # type: List[List[Cmp]]
        self._process_command_line = []  # type: List[List[Cmp]]
        self._process_guid = []  # type: List[List[Cmp]]
        self._process_id = []  # type: List[List[Cmp]]
        self._created_timestamp = []  # type: List[List[Cmp]]
        self._terminated_timestamp = []  # type: List[List[Cmp]]
        self._last_seen_timestamp = []  # type: List[List[Cmp]]

        # Edges
        self._parent = None  # type: Optional[PQ]
        self._bin_file = None  # type: Optional[FQ]
        self._children = None  # type: Optional[PQ]
        self._deleted_files = None  # type: Optional[FQ]

    def _get_var_block(self, binding_num, root, already_converted) -> str:
        if self in already_converted:
            return ""
        already_converted.add(self)

        filters = self._filters()

        parent = get_var_block(
            self._parent, "~children", binding_num, root, already_converted
        )

        children = get_var_block(
            self._children, "children", binding_num, root, already_converted
        )

        deleted_files = get_var_block(
            self._deleted_files, "deleted_files", binding_num, root, already_converted
        )

        block = f"""
            {filters} {{
                {parent}
                {children}
                {deleted_files}
            }}
            """

        return block

    def _get_var_block_root(self, binding_num, root, node_key):
        already_converted = {self}
        root_var = ""
        if self == root:
            root_var = f"Binding{binding_num} as "

        filters = self._filters()

        parent = get_var_block(
            self._parent, "~children", binding_num, root, already_converted
        )

        children = get_var_block(
            self._children, "children", binding_num, root, already_converted
        )

        deleted_files = get_var_block(
            self._deleted_files, "deleted_files", binding_num, root, already_converted
        )

        bin_file = get_var_block(
            self._bin_file, "bin_file", binding_num, root, already_converted
        )

        block = f"""
            {root_var} var(func: eq(node_key, "{node_key}")) {filters} {{
                {parent}
                {children}
                {deleted_files}
                {bin_file}
            }}
            """

        return block

    def get_neighbors(self) -> List[Any]:
        neighbors = [self._parent, self._bin_file, self._children, self._deleted_files]

        return [n for n in neighbors if n]

    def with_process_name(
        self,
        eq: Optional[Union[str, List]] = None,
        contains: Optional[Union[str, List]] = None,
        ends_with: Optional[Union[str, List]] = None,
    ):
        self._process_name.extend(_str_cmps("process_name", eq, contains, ends_with))
        return self

    def with_process_command_line(
        self,
        eq: Optional[Union[str, List]] = None,
        contains: Optional[Union[str, List]] = None,
        ends_with: Optional[Union[str, List]] = None,
    ):
        self._process_command_line.extend(
            _str_cmps("process_command_line", eq, contains, ends_with)
        )
        return self

    def with_process_guid(
        self,
        eq: Optional[Union[str, List]] = None,
        contains: Optional[Union[str, List]] = None,
        ends_with: Optional[Union[str, List]] = None,
    ):
        self._process_guid.extend(_str_cmps("process_guid", eq, contains, ends_with))
        return self

    def with_process_id(
        self,
        eq: Optional[Union[int, List]] = None,
        gt: Optional[Union[int, List]] = None,
        lt: Optional[Union[int, List]] = None,
    ):
        self._process_id.extend(_int_cmps("process_id", eq, gt, lt))
        return self

    def with_created_timestamp(
        self,
        eq: Optional[Union[int, List]] = None,
        gt: Optional[Union[int, List]] = None,
        lt: Optional[Union[int, List]] = None,
    ):
        self._created_timestamp.extend(_int_cmps("created_timestamp", eq, gt, lt))
        return self

    def with_terminated_timestamp(
        self,
        eq: Optional[Union[int, List]] = None,
        gt: Optional[Union[int, List]] = None,
        lt: Optional[Union[int, List]] = None,
    ):
        self._terminated_timestamp.extend(_int_cmps("terminated_timestamp", eq, gt, lt))
        return self

    def with_last_seen_timestamp(
        self,
        eq: Optional[Union[int, List]] = None,
        gt: Optional[Union[int, List]] = None,
        lt: Optional[Union[int, List]] = None,
    ):
        self._last_seen_timestamp.extend(_int_cmps("last_seen_timestamp", eq, gt, lt))
        return self

    def _filters(self) -> str:
        inner_filters = (
            _generate_filter(self._process_name),
            _generate_filter(self._process_command_line),
            _generate_filter(self._process_guid),
            _generate_filter(self._process_id),
        )

        inner_filters = [i for i in inner_filters if i]
        if not inner_filters:
            return ""
        return f"@filter({'AND'.join(inner_filters)})"

    def with_parent(self, process: PQ) -> PQ:
        process = deepcopy(process)
        process._children = self
        self._parent = process
        return self

    def with_bin_file(self, file: FQ) -> PQ:
        file = deepcopy(file)
        file._spawned_from = self
        self._bin_file = file
        return self

    def with_deleted_files(self, file: FQ) -> PQ:
        file = deepcopy(file)
        file._deleter = self
        self._deleted_files = file
        return self

    def with_children(self, children: PQ):
        children = deepcopy(children)
        children._parent = self
        self._children = children
        return self

    def _to_query(self, first):

        return ""

    def query_first(self, dgraph_client, contains_node_key=None) -> Optional[PV]:
        if contains_node_key:
            query = get_queries(self, node_key=contains_node_key)
        else:
            query = self._to_query(first=True)
        return None


class FileQuery(object):
    def __init__(self):
        # Attributes
        self._file_name = []  # type: List[List[Cmp]]
        self._file_path = []  # type: List[List[Cmp]]
        self._file_extension = []  # type: List[List[Cmp]]
        self._file_mime_type = []  # type: List[List[Cmp]]
        self._file_size = []  # type: List[List[Cmp]]
        self._file_version = []  # type: List[List[Cmp]]
        self._file_description = []  # type: List[List[Cmp]]
        self._file_product = []  # type: List[List[Cmp]]
        self._file_company = []  # type: List[List[Cmp]]
        self._file_directory = []  # type: List[List[Cmp]]
        self._file_inode = []  # type: List[List[Cmp]]
        self._file_hard_links = []  # type: List[List[Cmp]]
        self._md5_hash = []  # type: List[List[Cmp]]
        self._sha1_hash = []  # type: List[List[Cmp]]
        self._sha256_hash = []  # type: List[List[Cmp]]

        # Edges
        self._creator = None  # type: Optional[PQ]
        self._deleter = None  # type: Optional[PQ]
        self._writers = None  # type: Optional[PQ]
        self._readers = None  # type: Optional[PQ]
        self._spawned_from = None  # type: Optional[PQ]

    def with_file_name(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_name.extend(_str_cmps("file_name", eq, contains, ends_with))
        return self

    def with_file_path(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_path.extend(_str_cmps("file_path", eq, contains, ends_with))
        return self

    def with_file_extension(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_extension.extend(
            _str_cmps("file_extension", eq, contains, ends_with)
        )
        return self

    def with_file_mime_type(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_mime_type.extend(
            _str_cmps("file_mime_type", eq, contains, ends_with)
        )
        return self

    def with_file_size(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_size.extend(_str_cmps("file_size", eq, contains, ends_with))
        return self

    def with_file_version(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_version.extend(_str_cmps("file_version", eq, contains, ends_with))
        return self

    def with_file_description(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_description.extend(
            _str_cmps("file_description", eq, contains, ends_with)
        )
        return self

    def with_file_product(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_product.extend(_str_cmps("file_product", eq, contains, ends_with))
        return self

    def with_file_company(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_company.extend(_str_cmps("file_company", eq, contains, ends_with))
        return self

    def with_file_directory(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_directory.extend(
            _str_cmps("file_directory", eq, contains, ends_with)
        )
        return self

    def with_file_inode(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_inode.extend(_str_cmps("file_inode", eq, contains, ends_with))
        return self

    def with_file_hard_links(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._file_hard_links.extend(
            _str_cmps("file_hard_links", eq, contains, ends_with)
        )
        return self

    def with_md5_hash(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._md5_hash.extend(_str_cmps("md5_hash", eq, contains, ends_with))
        return self

    def with_sha1_hash(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._sha1_hash.extend(_str_cmps("sha1_hash", eq, contains, ends_with))
        return self

    def with_sha256_hash(self, eq=None, contains=None, ends_with=None) -> FQ:
        self._sha256_hash.extend(_str_cmps("sha256_hash", eq, contains, ends_with))
        return self

    def with_creator(self, creator: PQ) -> FQ:
        creator = deepcopy(creator)
        self._creator = creator
        return self

    def with_deleter(self, deleter: PQ) -> FQ:
        deleter = deepcopy(deleter)
        self._deleter = deleter
        deleter._deleted_files = self
        return self

    def with_writers(self, writers: PQ) -> FQ:
        writers = deepcopy(writers)
        self._writers = writers
        return self

    def with_readers(self, readers: PQ) -> FQ:
        readers = deepcopy(readers)
        self._readers = readers
        readers._read_files = self
        return self

    def _get_var_block(self, binding_num, root, already_converted) -> str:
        if self in already_converted:
            return ""
        already_converted.add(self)

        filters = self._filters()

        creator = get_var_block(
            self._creator, "~created_files", binding_num, root, already_converted
        )

        deleter = get_var_block(
            self._deleter, "~deleted_files", binding_num, root, already_converted
        )

        writers = get_var_block(
            self._writers, "~wrote_files", binding_num, root, already_converted
        )

        readers = get_var_block(
            self._readers, "~read_files", binding_num, root, already_converted
        )

        block = f"""
            {filters} {{
                {creator}
                {deleter}
                {writers}
                {readers}
            }}
            """

        return block

    def _get_var_block_root(self, binding_num, root, node_key):
        already_converted = {self}
        root_var = ""
        if self == root:
            root_var = f"Binding{binding_num} as "

        filters = self._filters()

        creator = get_var_block(
            self._creator, "~created_files", binding_num, root, already_converted
        )

        deleter = get_var_block(
            self._deleter, "~deleted_files", binding_num, root, already_converted
        )

        writers = get_var_block(
            self._writers, "~wrote_files", binding_num, root, already_converted
        )

        readers = get_var_block(
            self._readers, "~read_files", binding_num, root, already_converted
        )

        spawned_from = get_var_block(
            self._spawned_from, "~bin_file", binding_num, root, already_converted
        )

        block = f"""
            {root_var} var(func: eq(node_key, "{node_key}")) {filters} {{
                {creator}
                {deleter}
                {writers}
                {readers}
                {spawned_from}
            }}
            """

        return block

    def _filters(self) -> str:
        inner_filters = (
            _generate_filter(self._file_name),
            _generate_filter(self._file_path),
            _generate_filter(self._file_extension),
            _generate_filter(self._file_mime_type),
            _generate_filter(self._file_size),
            _generate_filter(self._file_version),
            _generate_filter(self._file_description),
            _generate_filter(self._file_product),
            _generate_filter(self._file_company),
            _generate_filter(self._file_directory),
            _generate_filter(self._file_inode),
            _generate_filter(self._file_hard_links),
            _generate_filter(self._md5_hash),
            _generate_filter(self._sha1_hash),
            _generate_filter(self._sha256_hash),
        )

        inner_filters = [i for i in inner_filters if i]
        if not inner_filters:
            return ""

        return f"@filter({'AND'.join(inner_filters)})"

    def get_neighbors(self) -> List[Any]:
        neighbors = [
            self._creator,
            self._deleter,
            self._writers,
            self._readers,
            self._spawned_from,
        ]

        return [n for n in neighbors if n]


import unittest
import re


class Test(unittest.TestCase):
    @staticmethod
    def format_query(query):
        return re.sub(" +", " ", (query.replace("\t", "").replace("\n", "").strip()))

        # return (
        #     query
        #     .replace("\t", "")
        #     .replace("\n", "")# )

    def test_any_process_key_opt(self):
        p = ProcessQuery()
        queries = self.format_query(get_queries(p, node_key="keyA"))

        expected = self.format_query(
            """
            {
                Binding0 as var(func: eq(node_key, "keyA")) { }
                res(func: uid(Binding0), first: 1) {
                    expand(_all_) {}
                }
            }"""
        )
        assert queries == expected, "\n" + queries + "\n" + expected

    def test_has_process_name(self):
        ProcessQuery().with_process_name()
        p = ProcessQuery()
        queries = self.format_query(get_queries(p, node_key="keyA"))

        expected = self.format_query(
            """
        {
            Binding0 as var(func: eq(node_key, "keyA")) { }
            res(func: uid(Binding0), first: 1) {
                expand(_all_) {} 
            }
        }
        """
        )
        assert queries == expected, "\n" + queries + "\n" + expected

    def test_has_bin_file(self):

        p = ProcessQuery().with_bin_file(FileQuery())
        queries = self.format_query(get_queries(p, node_key="keyA"))

        expected = self.format_query(
            """
        {
            Binding0 as var(func: eq(node_key, "keyA")) {
                bin_file { }
            }
            
            var(func: eq(node_key, "keyA")) {
                Binding1 as ~bin_file { }
            }
            res(func: uid(Binding0, Binding1), first: 1) {
                expand(_all_) {expand(_all_) {}} 
            }
        }
        """
        )
        assert queries == expected, "\n" + queries + "\n" + expected

    def test_has_bin_file_with_path(self):

        p = ProcessQuery().with_bin_file(FileQuery().with_file_path(eq="foo"))
        queries = self.format_query(get_queries(p, node_key="keyA"))

        expected = self.format_query(
            """
        {
            Binding0 as var(func: eq(node_key, "keyA")) {
                bin_file @filter(((eq(file_path, "foo")))) { }
            }
            var(func: eq(node_key, "keyA")) @filter(((eq(file_path, "foo"))))  {
                Binding1 as ~bin_file { }
            }
            
            res(func: uid(Binding0, Binding1), first: 1) {
                expand(_all_) {expand(_all_) {}} 
            }
        }
        """
        )
        assert queries == expected, "\n" + queries + "\n" + expected


if __name__ == "__main__":
    unittest.main()
    # main()
