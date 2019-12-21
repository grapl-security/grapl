import abc
from collections import defaultdict
from typing import Union, List, Tuple, Sequence, Type, Set, DefaultDict

from typing_extensions import Literal


StrIndex = Union[Literal["trigram"], Literal["exact"], Literal["hash"]]


def format(s: str, indent: int = 4, cur_indent: int = 2, output: str = "") -> str:
    if not s:
        return output
    nl_index = s.find("\n")
    # print('ix', nl_index)

    if nl_index == -1:
        nl_index = len(s)

    line = s[:nl_index].strip()
    if not line:
        return format(s[nl_index + 1 :], indent, cur_indent, output=output)

    if "}" in line:
        cur_indent -= indent

    space_buf = " " * cur_indent

    if "{" in line:
        cur_indent += indent

    output = output + space_buf + line + "\n"
    return format(s[nl_index + 1 :], indent, cur_indent, output=output)


class NodeSchema(abc.ABC):
    def __init__(self) -> None:
        self.node_type = self.self_type()
        self.str_props = []  # type: List[Tuple[str, Sequence[StrIndex]]]
        self.int_props = []  # type: List[str]
        self.bool_props = []  # type: List[str]
        self.forward_edges = []  # type: List[Tuple[str, UidType, str]]
        self.reverse_edges = []  # type: List[Tuple[str, UidType, str]]

    @staticmethod
    @abc.abstractmethod
    def self_type() -> str:
        pass

    def with_str_prop(
        self, prop_name: str, indexes: Sequence[StrIndex] = ()
    ) -> "NodeSchema":
        if indexes is ():
            indexes = ["trigram", "exact", "hash"]
        self.str_props.append((prop_name, indexes))
        return self

    def with_int_prop(self, prop_name: str) -> "NodeSchema":
        self.int_props.append(prop_name)
        return self

    def with_bool_prop(self, prop_name: str) -> "NodeSchema":
        self.bool_props.append(prop_name)
        return self

    def with_forward_edge(
        self, edge_name: str, edge: "UidType", reverse_name: str
    ) -> "NodeSchema":
        self.forward_edges.append((edge_name, edge, reverse_name))
        return self

    def with_reverse_edge(
        self, reverse_name: str, edge: "UidType", forward_name: str
    ) -> "NodeSchema":
        self.reverse_edges.append((reverse_name, edge, forward_name))
        return self

    def generate_type(self) -> str:
        str_types = ""
        int_types = ""
        bool_types = ""
        edge_types = ""

        for prop_name, _indexes in self.str_props:
            str_types += f"{prop_name}: string\n"

        for prop_name in self.int_props:
            int_types += f"{prop_name}: int\n"

        for prop_name in self.bool_props:
            bool_types += f"{prop_name}: bool\n"

        for prop_name, edge_type, reverse_name in self.forward_edges:

            if isinstance(edge_type, list):
                type_name = edge_type[0]._inner_type.self_type()
                edge_types += f"{prop_name}: [uid]  # type: {type_name}\n"
            else:
                type_name = edge_type._inner_type.self_type()
                edge_types += f"{prop_name}: uid  # type: {type_name}\n"

        type_def = f"""
            type {self.node_type} {{
                node_key: string
                {str_types}
                {int_types}
                {bool_types}
                {edge_types}
            
            }}
        """

        formatted_typedef = format(type_def)
        assert formatted_typedef
        return formatted_typedef

    def to_schema_str(self) -> str:
        _str_prop_schema = []
        for prop_name, indexes in self.str_props:
            fmt_indexes = ", ".join(indexes)
            _str_prop_schema.append(f"{prop_name}: string @index({fmt_indexes}) .\n")

        str_prop_schema = "\n".join(_str_prop_schema)

        int_prop_schema = ""
        for prop_name in self.int_props:
            int_prop_schema += f"{prop_name}: int @index(int) .\n"

        bool_prop_schema = ""
        for prop_name in self.bool_props:
            bool_prop_schema += f"{prop_name}: bool @index(bool) .\n"

        edge_prop_schema = ""
        for f_edge_name, edge_type, _r_edge_name in self.forward_edges:
            if isinstance(edge_type, OneToMany) or isinstance(edge_type, ManyToMany):
                edge_prop_schema += f"{f_edge_name}: [uid] @reverse .\n"
            else:
                edge_prop_schema += f"{f_edge_name}: uid @reverse .\n"

        schema = f"""
            node_key: string @upsert @index(hash) .
            
            {str_prop_schema}
            {int_prop_schema} 
            {bool_prop_schema} 
            {edge_prop_schema}        
        """.replace(
            "  ", ""
        )

        return format(schema)


class ManyToOne(object):
    def __init__(self, inner_type: Type[NodeSchema]):
        self._inner_type = inner_type


class ManyToMany(object):
    def __init__(self, inner_type: Type[NodeSchema]):
        self._inner_type = inner_type


class OneToMany(object):
    def __init__(self, inner_type: Type[NodeSchema]):
        self._inner_type = inner_type


class OneToOne(object):
    def __init__(self, inner_type: Type[NodeSchema]):
        self._inner_type = inner_type


UidType = Union[ManyToOne, ManyToMany, OneToMany, OneToOne]


def generate_with_str_prop_method(node_type: str, prop_name: str) -> str:
    return f"""
    def with_{prop_name}(
            self,
            eq: Optional[StrCmp] = None,
            contains: Optional[StrCmp] = None,
            ends_with: Optional[StrCmp] = None,
            starts_with: Optional[StrCmp] = None,
            regexp: Optional[StrCmp] = None,
            distance: Optional[Tuple[StrCmp, int]] = None,
    ) -> 'NQ':
        self.set_str_property_filter(
            "{prop_name}", _str_cmps(
                "{prop_name}",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self
    """


def generate_with_int_prop_method(node_type: str, prop_name: str) -> str:
    return f"""
    def with_{prop_name}(
            self: 'NQ',
            eq: Optional['IntCmp'] = None,
            gt: Optional['IntCmp'] = None,
            lt: Optional['IntCmp'] = None,
    ) -> 'NQ':
        self.set_int_property_filter(
            "{prop_name}", _int_cmps("{prop_name}", eq=eq, gt=gt, lt=lt)
        )
        return self
    """


def generate_with_f_edge_method(
    node_type: str, f_edge_name: str, r_edge_name: str, edge_type: Union[UidType]
) -> str:
    edge_type_str = f"{edge_type._inner_type.self_type()}Query"

    return f"""\
    def with_{f_edge_name}(
            self: 'NQ',
            {f_edge_name}_query: Optional['I{edge_type_str}'] = None
    ) -> 'NQ':
        {f_edge_name} = {f_edge_name}_query or {edge_type_str}()

        self.set_forward_edge_filter("{f_edge_name}", {f_edge_name})
        {f_edge_name}.set_reverse_edge_filter("~{f_edge_name}", self, "{f_edge_name}")
        return self        
        """


def generate_imports(schema: NodeSchema) -> str:
    imports = """\
from typing import *

from grapl_analyzerlib.nodes.types import PropertyT
from grapl_analyzerlib.nodes.viewable import EdgeViewT, ForwardEdgeView
from grapl_analyzerlib.nodes.comparators import Cmp, IntCmp, _int_cmps, StrCmp, _str_cmps

from pydgraph import DgraphClient
    """
    return imports


def generate_plugin_query(plugin_schema: NodeSchema) -> str:
    query_type = f"{plugin_schema.self_type()}Query"
    view_type = f"{plugin_schema.self_type()}View"

    int_query_cmps = ""

    for int_prop in plugin_schema.int_props:
        cmp = f"        self._{int_prop} = []  # type: List[List[Cmp[int]]]\n"
        int_query_cmps += cmp

    str_query_cmps = ""

    for str_prop in plugin_schema.str_props:
        cmp = f"        self._{str_prop[0]} = []  # type: List[List[Cmp[str]]]\n"
        str_query_cmps += cmp

    f_edge_query_cmps = ""

    for f_edge in plugin_schema.forward_edges:
        edge_name = f_edge[0]
        edge_type = f"{f_edge[1]._inner_type.self_type()}Query"

        cmp = f"        self._{edge_name} = None  # type: Optional[I{edge_type}]\n"
        f_edge_query_cmps += cmp

    str_methods = ""
    for str_prop in plugin_schema.str_props:
        method = generate_with_str_prop_method(plugin_schema.self_type(), str_prop[0])
        str_methods += method + "\n"

    int_methods = ""
    for int_prop in plugin_schema.int_props:
        method = generate_with_int_prop_method(plugin_schema.self_type(), int_prop)
        int_methods += method + "\n"

    f_edge_methods = ""
    for f_edge in plugin_schema.forward_edges:
        method = generate_with_f_edge_method(
            plugin_schema.self_type(), f_edge[0], f_edge[2], f_edge[1]
        )
        f_edge_methods += method + "\n"

    query = f"""
{generate_imports(plugin_schema)}

I{query_type} = TypeVar('I{query_type}', bound='{query_type}')

class {query_type}(DynamicNodeQuery):
    def __init__(self):
        super({query_type}, self).__init__('{plugin_schema.self_type()}',{view_type})
"""
    query += f"{int_query_cmps}"
    query += f"\n"
    query += f"{str_query_cmps}"
    query += f"\n"
    query += f"{f_edge_query_cmps}"
    query += f"\n"
    query += f"{str_methods}"
    query += f"\n"
    query += f"{int_methods}"
    query += f"\n"
    query += f"{f_edge_methods}"

    return query


def plugin_view_init(plugin_schema: NodeSchema) -> str:
    spaces = "            "

    args = ""
    for prop_name in plugin_schema.int_props:
        args += spaces + f"{prop_name}: Optional[int] = None,\n"

    for prop_name, _indices in plugin_schema.str_props:
        args += spaces + f"{prop_name}: Optional[str] = None,\n"

    for f_edge_name, edge_type, r_edge_name in plugin_schema.forward_edges:
        if isinstance(edge_type, OneToOne) or isinstance(edge_type, ManyToOne):
            args += (
                spaces
                + f"{f_edge_name}:' Optional[{edge_type._inner_type.self_type()}View]' = None,\n"
            )

        elif isinstance(edge_type, OneToMany) or isinstance(edge_type, ManyToMany):
            args += (
                spaces
                + f"{f_edge_name}: 'Optional[List[{edge_type._inner_type.self_type()}View]]' = None,\n"
            )

    spaces = "        "
    self_assignments = ""

    for prop_name in plugin_schema.int_props:
        self_assignments += spaces + f"self.{prop_name} = {prop_name}\n"

    for prop_name, _indices in plugin_schema.str_props:
        self_assignments += spaces + f"self.{prop_name} = {prop_name}\n"

    for f_edge_name, edge_type, r_edge_name in plugin_schema.forward_edges:
        self_assignments += spaces + f"self.{f_edge_name} = {f_edge_name}\n"

    query = f"""
    
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: str,
            node_type: str,
{args}
    ):
        super({plugin_schema.self_type()}View, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid, node_type=node_type
        )
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.node_type = node_type

{self_assignments}
    
    """

    return query


def generate_plugin_view_get_methods(plugin_schema: NodeSchema) -> str:

    methods = ""

    for prop_name in plugin_schema.int_props:
        method = f"""
    def get_{prop_name}(self) -> Optional[int]:
        if not self.{prop_name}:
            self.{prop_name} = cast(Optional[int], self.fetch_property("{prop_name}", int))
        return self.{prop_name}
        """
        methods += method

    for prop_name, _indices in plugin_schema.str_props:
        method = f"""
    def get_{prop_name}(self) -> Optional[str]:
        if not self.{prop_name}:
            self.{prop_name} = cast(Optional[str], self.fetch_property("{prop_name}", str))
        return self.{prop_name}
        """
        methods += method

    # for f_edge_name, edge_type, r_edge_name in plugin_schema.forward_edges:
    #     self_assignments += spaces + f"self.{f_edge_name} = {f_edge_name}\n"

    return methods


def generate_get_property_types(plugin_schema: NodeSchema) -> str:
    spaces = "                "
    property_types = []
    for prop_name in plugin_schema.int_props:
        property_types.append(spaces + f"'{prop_name}': int,")

    for prop_name, _indices in plugin_schema.str_props:
        property_types.append(spaces + f"'{prop_name}': str,")

    formatted = "\n".join(property_types)

    method = f"""
    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {{
{formatted}
        }}

    """
    return method


def generate_get_forward_edge_types(plugin_schema: NodeSchema) -> str:
    spaces = "                "

    f_edges = []

    for f_edge_name, edge_type, r_edge_name in plugin_schema.forward_edges:
        edge_type_name = edge_type._inner_type.self_type()
        if isinstance(edge_type, OneToOne) or isinstance(edge_type, ManyToOne):
            f_edges.append(spaces + f"'{f_edge_name}': {edge_type_name}View,")
        elif isinstance(edge_type, OneToMany) or isinstance(edge_type, ManyToMany):
            f_edges.append(spaces + f"'{f_edge_name}': [{edge_type_name}View],")

    formatted = "\n".join(f_edges)
    query = f"""

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        f_edges = {{
{formatted}
        }}  # type: Dict[str, Optional["EdgeViewT"]]

        return cast(Mapping[str, "EdgeViewT"], {{
            fe[0]: fe[1] for fe in f_edges.items() if fe[1]
        }})
    """
    return query


def generate_get_forward_edges(plugin_schema: NodeSchema) -> str:
    spaces = "                "

    f_edges = []

    for f_edge_name, _edge_type, _r_edge_name in plugin_schema.forward_edges:
        f_edges.append(spaces + f"'{f_edge_name}': self.{f_edge_name},")

    formatted = "\n".join(f_edges)

    query = f"""
    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {{
{formatted}
        }}  # type: Dict[str, Optional[ForwardEdgeView]]

        return cast(
            "Mapping[str, ForwardEdgeView]",   
            {{fe[0]: fe[1] for fe in f_edges.items() if fe[1]}}
        )
    """

    return query


def generate_get_properties(plugin_schema: NodeSchema) -> str:
    spaces = "                "
    properties = []
    for prop_name in plugin_schema.int_props:
        properties.append(spaces + f"'{prop_name}': self.{prop_name},")

    for prop_name, _indices in plugin_schema.str_props:
        properties.append(spaces + f"'{prop_name}': self.{prop_name},")

    formatted = "\n".join(properties)

    query = f"""
    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        props = {{
{formatted}
        }}

        return {{p[0]: p[1] for p in props.items() if p[1] is not None}}
    """

    return query


def generate_plugin_view(plugin_schema: NodeSchema) -> str:
    view_type = f"{plugin_schema.self_type()}View"
    query = f"I{view_type} = TypeVar('I{view_type}', bound='{view_type}')\n\n"
    query += f"class {view_type}(DynamicNodeView):"
    query += plugin_view_init(plugin_schema)
    query += generate_plugin_view_get_methods(plugin_schema)
    query += generate_get_property_types(plugin_schema)
    query += generate_get_forward_edge_types(plugin_schema)
    query += generate_get_forward_edges(plugin_schema)
    query += generate_get_properties(plugin_schema)

    return query


def generate_query_extension_method(
    self_type: str, extension_name: str, f_edge_name: str, r_edge_name: str
) -> str:

    method = f"""\
    def with_{r_edge_name}(
        self: 'NQ',
        {r_edge_name}_query: Optional['I{self_type}Query'] = None
    ) -> 'NQ':
        {r_edge_name} = {r_edge_name}_query or {self_type}Query()
        {r_edge_name}.with_{f_edge_name}(
            cast({extension_name}, self)
        )

        return self
    """

    return method


def generate_plugin_query_extensions(plugin_schema: NodeSchema) -> str:
    extended_types = defaultdict(
        set
    )  # type: DefaultDict[Type[NodeSchema], Set[Tuple[str, UidType, str]]]

    for f_edge_name, edge_type, r_edge_name in plugin_schema.forward_edges:
        extended_type = edge_type._inner_type

        extended_types[extended_type].add((f_edge_name, edge_type, r_edge_name))

    extensions = []

    for extended_type in extended_types.keys():
        extension = f"class {plugin_schema.self_type()}"
        extension += f"Extends{extended_type.self_type()}Query"
        extension += f"({extended_type.self_type()}Query):"

        for f_edge_name, edge_type, r_edge_name in extended_types[extended_type]:
            extension_method = generate_query_extension_method(
                plugin_schema.self_type(),
                extended_type.self_type() + "Query",
                f_edge_name,
                r_edge_name,
            )

            extension += "\n" + extension_method

        extensions.append(extension)

    return "\n".join(extensions)


def generate_view_extension_method(
    self_type: str, extension_name: str, f_edge_name: str, r_edge_name: str
) -> str:

    method = f"""\
    def get_{r_edge_name}(
        self,
    ) -> '{self_type}View':
        return cast({self_type}View, self.fetch_edge("~{f_edge_name}", {self_type}View))
    """

    return method


def generate_plugin_view_extensions(plugin_schema: NodeSchema) -> str:
    extended_types = defaultdict(
        set
    )  # type: DefaultDict[Type[NodeSchema], Set[Tuple[str, UidType, str]]]

    for f_edge_name, edge_type, r_edge_name in plugin_schema.forward_edges:
        extended_type = edge_type._inner_type

        extended_types[extended_type].add((f_edge_name, edge_type, r_edge_name))

    extensions = []

    for extended_type in extended_types.keys():
        extension = f"class {plugin_schema.self_type()}"
        extension += f"Extends{extended_type.self_type()}View"
        extension += f"({extended_type.self_type()}View):"

        for f_edge_name, edge_type, r_edge_name in extended_types[extended_type]:
            extension_method = generate_view_extension_method(
                plugin_schema.self_type(),
                extended_type.self_type() + "View",
                f_edge_name,
                r_edge_name,
            )

            extension += "\n" + extension_method

        extensions.append(extension)

    return "\n".join(extensions)


# def main() -> None:
#     from grapl_analyzerlib.schemas.process_schema import ProcessSchema
#     class AuidSchema(NodeSchema):
#         def __init__(self):
#             super(AuidSchema, self).__init__()
#             (
#                 self
#                 .with_int_prop("auid")
#             )
#
#         @staticmethod
#         def self_type() -> str:
#             return "Auid"
#
#     class AuidAssumptionSchema(NodeSchema):
#         def __init__(self):
#             super(AuidAssumptionSchema, self).__init__()
#             (
#                 self.with_int_prop("assumed_timestamp")
#                     .with_int_prop("assuming_process_id")
#                     .with_forward_edge("assumed_auid", ManyToOne(AuidSchema), 'auid_assumptions')
#                     .with_forward_edge("assuming_process", OneToOne(ProcessSchema), 'assumed_auid')
#             )
#
#         @staticmethod
#         def self_type() -> str:
#             return "AuidAssumption"
#
#
#     auid_assumption_schema = AuidAssumptionSchema()
#
#     auid_assumption_query = generate_plugin_query(auid_assumption_schema)
#     auid_assumption_view = generate_plugin_view(auid_assumption_schema)
#     auid_assumption_query_extensions = generate_plugin_query_extensions(auid_assumption_schema)
#     auid_assumption_view_extensions = generate_plugin_view_extensions(auid_assumption_schema)
#
#     auid_schema = AuidSchema()
#     auid_query = generate_plugin_query(auid_schema)
#     auid_view = generate_plugin_view(auid_schema)
#     auid_query_extensions = generate_plugin_query_extensions(auid_schema)
#     auid_view_extensions = generate_plugin_view_extensions(auid_schema)
#
#     print(auid_query)
#     print(auid_view)
#     print(auid_query_extensions)
#     print(auid_view_extensions)
#
#
#     print(auid_assumption_query)
#     print(auid_assumption_view)
#
#     print(auid_assumption_query_extensions)
#     print(auid_assumption_view_extensions)
#
#
# if __name__ == '__main__':
#     main()
