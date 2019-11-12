from typing import Union, List, Tuple, Sequence
from typing_extensions import Literal

StrIndex = Union[
    Literal["trigram"],
    Literal["exact"],
    Literal["hash"],
]


class DynamicNodeSchema(object):
    def __init__(self) -> None:
        self.str_props = []  # type: List[Tuple[str, Sequence[StrIndex]]]
        self.int_props = []  # type: List[str]
        self.forward_edges = []  # type: List[str]

    def with_str_prop(
            self,
            prop_name: str,
            indexes: Sequence[StrIndex] = ()
    ) -> 'DynamicNodeSchema':
        if indexes is ():
            indexes = ["trigram", "exact", "hash"]
        self.str_props.append((prop_name, indexes))
        return self

    def with_int_prop(self, prop_name: str) -> 'DynamicNodeSchema':
        self.int_props.append(prop_name)
        return self

    def with_forward_edge(self, edge_name: str) -> 'DynamicNodeSchema':
        self.forward_edges.append(edge_name)
        return self

    def to_schema_str(self, engagement: bool) -> str:
        str_prop_schema = ""
        for prop_name, indexes in self.str_props:
            fmt_indexes = ", ".join(indexes)
            str_prop_schema += f"{prop_name}: string @index({fmt_indexes}) .\n"

        int_prop_schema = ""
        for prop_name in self.int_props:
            int_prop_schema += f"{prop_name}: string @index(int) .\n"

        edge_prop_schema = ""
        for edge_name in self.forward_edges:
            edge_prop_schema += f"{edge_name}: uid @reverse .\n"

        schema = f"""
            node_key: string @upsert @index(hash) .
            node_type: string @index(hash) .
            
            {str_prop_schema}
            {int_prop_schema} 
            {edge_prop_schema}        
        """.replace("  ", "").replace("\n\n\n", "\n\n").strip()

        if engagement:
            schema += "\n"
            schema += "risks: uid @reverse ."

        return schema


if __name__ == '__main__':

    ipc_schema = (
        DynamicNodeSchema()
        .with_str_prop('ipc_type', ["hash"])
        .with_int_prop('src_pid')
        .with_int_prop('dst_pid')
        .with_forward_edge('ipc_creator')
        .with_forward_edge('ipc_recipient')
    )

    print(ipc_schema.to_schema_str(False))
    # print(ipc_schema.to_schema_str(True))
