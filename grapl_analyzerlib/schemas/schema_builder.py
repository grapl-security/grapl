import abc
from typing import Union, List, Tuple, Sequence, Type

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
        self.forward_edges = []  # type: List[Tuple[str, UidType]]

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

    def with_forward_edge(self, edge_name: str, edge: 'UidType') -> 'NodeSchema':
        self.forward_edges.append((edge_name, edge))
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

        for prop_name, edge_type in self.forward_edges:

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


UidType = Union[Type[NodeSchema], List[Type[NodeSchema]]]
