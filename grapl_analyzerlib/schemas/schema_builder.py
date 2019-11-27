import abc
from typing import Union, List, Tuple, Sequence, Type, NewType, TypeVar

from typing_extensions import Literal


StrIndex = Union[
    Literal["trigram"],
    Literal["exact"],
    Literal["hash"],
]


def format(s: str, indent: int = 4, cur_indent: int = 2, output: str = "") -> str:
    if not s:
        return output
    nl_index = s.find('\n')
    # print('ix', nl_index)

    if nl_index == -1:
        nl_index = len(s)

    line = s[:nl_index].strip()
    if not line:
        return format(s[nl_index + 1:], indent, cur_indent, output=output)

    if "}" in line:
        cur_indent -= indent

    space_buf = " " * cur_indent

    if "{" in line:
        cur_indent += indent

    output = output + space_buf + line + "\n"
    return format(s[nl_index + 1:], indent, cur_indent, output=output)


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
            self,
            prop_name: str,
            indexes: Sequence[StrIndex] = ()
    ) -> 'NodeSchema':
        if indexes is ():
            indexes = ["trigram", "exact", "hash"]
        self.str_props.append((prop_name, indexes))
        return self

    def with_int_prop(self, prop_name: str) -> 'NodeSchema':
        self.int_props.append(prop_name)
        return self

    def with_bool_prop(self, prop_name: str) -> 'NodeSchema':
        self.bool_props.append(prop_name)
        return self

    def with_forward_edge(self, edge_name: str, edge: 'UidType', reverse_name: str) -> 'NodeSchema':
        self.forward_edges.append((edge_name, edge, reverse_name))
        return self

    def with_reverse_edge(self, reverse_name: str, edge: 'UidType', forward_name: str) -> 'NodeSchema':
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
                type_name = edge_type[0].self_type()
                edge_types += f"{prop_name}: [uid]  # type: {type_name}\n"
            else:
                type_name = edge_type.self_type()
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
        for edge_name, edge_type in self.forward_edges:
            if isinstance(edge_type, list):
                edge_prop_schema += f"{edge_name}: [uid] @reverse .\n"
            else:
                edge_prop_schema += f"{edge_name}: uid @reverse .\n"

        schema = f"""
            node_key: string @upsert @index(hash) .
            
            {str_prop_schema}
            {int_prop_schema} 
            {bool_prop_schema} 
            {edge_prop_schema}        
        """.replace("  ", "")

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


def generate_with_str_prop_method(
        node_type: str,
        prop_name: str,
) -> str:
    return f"""
    def with_{prop_name}(
            self,
            eq: Optional['StrCmp'] = None,
            contains: Optional['StrCmp'] = None,
            ends_with: Optional['StrCmp'] = None,
    ) -> '{node_type}Query':
        self.set_str_property_filter(
            "{prop_name}", _str_cmps("{prop_name}", eq=eq, gt=gt, lt=lt)
        )
        return self
    """


def generate_with_int_prop_method(
        node_type: str,
        prop_name: str,
) -> str:
    return f"""
    def with_{prop_name}(
            self,
            eq: Optional['IntCmp'] = None,
            gt: Optional['IntCmp'] = None,
            lt: Optional['IntCmp'] = None,
    ) -> '{node_type}Query':
        self.set_int_property_filter(
            "{prop_name}", _int_cmps("{prop_name}", eq=eq, gt=gt, lt=lt)
        )
        return self
    """


def generate_with_f_edge_method(
        node_type: str,
        f_edge_name: str,
        r_edge_name: str,
        edge_type: Union[UidType],
) -> str:
    edge_type_str = f"{edge_type._inner_type.self_type()}Query"

    return f"""
    def with_{f_edge_name}(
            self,
            {f_edge_name}_query: Optional['{edge_type_str}'] = None
    ) -> '{node_type}Query':
        {f_edge_name} = {f_edge_name}_query or {edge_type_str}()

        self.set_forward_edge_filter("{f_edge_name}", {f_edge_name})
        {f_edge_name}.set_reverse_edge_filter("~{f_edge_name}", self, "{f_edge_name}")
        return self        
        """


def main() -> None:
    from grapl_analyzerlib.schemas import FileSchema
    from grapl_analyzerlib.schemas import ProcessSchema
    class AuidSchema(NodeSchema):
        def __init__(self):
            super(AuidSchema, self).__init__()
            (
                self
                .with_int_prop("auid")
            )

        @staticmethod
        def self_type() -> str:
            return "Auid"

    class AuidAssumptionSchema(NodeSchema):
        def __init__(self):
            super(AuidAssumptionSchema, self).__init__()
            (
                self.with_int_prop("assumed_timestamp")
                    .with_int_prop("assuming_process_id")
                    .with_forward_edge("assumed_auid", ManyToOne(AuidSchema), 'assumptions')
                    .with_forward_edge("assuming_process", OneToOne(ProcessSchema), 'assumed_auid')
            )

        @staticmethod
        def self_type() -> str:
            return "AuidAssumption"


    p = AuidAssumptionSchema()

    query_type = f"{p.self_type()}Query"
    view_type = f"{p.self_type()}View"

    int_query_cmps = ""

    for int_prop in p.int_props:
        cmp = f"        self._{int_prop} = []  # type: List[List[Cmp[int]]]\n"
        int_query_cmps += cmp

    str_query_cmps = ""

    for str_prop in p.str_props:
        cmp = f"        self._{str_prop[0]} = []  # type: List[List[Cmp[str]]]\n"
        str_query_cmps += cmp

    f_edge_query_cmps = ""

    for f_edge in p.forward_edges:
        edge_name = f_edge[0]
        edge_type = f"{f_edge[1]._inner_type.self_type()}Query"

        cmp = f"        self._{edge_name} = None  # type: 'Optional[{edge_type}]'\n"
        f_edge_query_cmps += cmp

    str_methods = ""
    for str_prop in p.str_props:
        method = generate_with_str_prop_method(p.self_type(), str_prop[0])
        str_methods += method + "\n"

    int_methods = ""
    for int_prop in p.int_props:
        method = generate_with_int_prop_method(p.self_type(), int_prop)
        int_methods += method + "\n"

    f_edge_methods = ""
    for f_edge in p.forward_edges:
        method = generate_with_f_edge_method(p.self_type(), f_edge[0], f_edge[2], f_edge[1])
        f_edge_methods += method + "\n"

    query = f"""
class {query_type}(Queryable):
    def __init__(self):
        super({query_type}, self).__init__({view_type})
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

    print(query)

if __name__ == '__main__':
    main()