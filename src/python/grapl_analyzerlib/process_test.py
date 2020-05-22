import json
from copy import deepcopy
from typing import *

from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzerlib.nodes.comparators import IntCmp, _str_cmps, StrCmp, _int_cmps
from grapl_analyzerlib.nodes.dynamic_node import DynamicNodeQuery, DynamicNodeView
from grapl_analyzerlib.nodes.process_node import ProcessQuery, ProcessView, IProcessView
from grapl_analyzerlib.nodes.types import Property, PropertyT
from grapl_analyzerlib.nodes.viewable import Viewable, EdgeViewT, ForwardEdgeView

T = TypeVar("T")


def create_edge(
    client: DgraphClient, from_uid: str, edge_name: str, to_uid: str
) -> None:
    if edge_name[0] == "~":
        mut = {"uid": to_uid, edge_name[1:]: {"uid": from_uid}}

    else:
        mut = {"uid": from_uid, edge_name: {"uid": to_uid}}

    txn = client.txn(read_only=False)
    try:
        txn.mutate(set_obj=mut, commit_now=True)
    finally:
        txn.discard()


def _upsert(client: DgraphClient, node_dict: Dict[str, Property]) -> str:
    if node_dict.get("uid"):
        node_dict.pop("uid")
    node_dict["uid"] = "_:blank-0"
    node_key = node_dict["node_key"]
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}")) {{
                    uid,  
                    expand(_all_)            
            }}
        }}
        """
    txn = client.txn(read_only=False)

    try:
        res = json.loads(txn.query(query).json)["q0"]
        new_uid = None
        if res:
            node_dict["uid"] = res[0]["uid"]
            new_uid = res[0]["uid"]

        mutation = node_dict

        m_res = txn.mutate(set_obj=mutation, commit_now=True)
        uids = m_res.uids

        if new_uid is None:
            new_uid = uids["blank-0"]
        return str(new_uid)

    finally:
        txn.discard()


def upsert(
    client: DgraphClient,
    type_name: str,
    view_type: Type[Viewable],
    node_key: str,
    node_props: Dict[str, Property],
) -> Viewable:
    node_props["node_key"] = node_key
    node_props["dgraph.type"] = type_name
    uid = _upsert(client, node_props)
    # print(f'uid: {uid}')
    node_props["uid"] = uid
    # print(node_props['node_key'])
    return view_type.from_dict(client, node_props)


def main() -> None:
    local_client = DgraphClient(DgraphClientStub("localhost:9080"))

    parent = {
        "process_id": 100,
        "process_name": "word.exe",
    }  # type: Dict[str, Property]

    child = {"process_id": 1234, "process_name": "cmd.exe"}  # type: Dict[str, Property]

    parent_view = upsert(
        local_client,
        "Process",
        ProcessView,
        "ea75f056-61a1-479d-9ca2-f632d2c67205",
        parent,
    )

    child_view = upsert(
        local_client,
        "Process",
        ProcessView,
        "10f585c2-cf31-41e2-8ca5-d477e78be3ac",
        child,
    )

    create_edge(local_client, parent_view.uid, "children", child_view.uid)

    queried_child_0 = ProcessQuery().with_process_id(eq=1234).query_first(local_client)

    assert queried_child_0
    assert queried_child_0.node_key == child_view.node_key

    queried_child_1 = (
        ProcessQuery()
        .with_process_id(eq=1234)
        .query_first(
            local_client, contains_node_key="10f585c2-cf31-41e2-8ca5-d477e78be3ac"
        )
    )

    assert queried_child_1
    assert queried_child_1.node_key == child_view.node_key
    assert queried_child_1.node_key == queried_child_0.node_key

    p = (
        ProcessQuery().with_parent().query_first(local_client)
    )  # type: Optional[ProcessView]

    assert p

    p.get_parent()


if __name__ == "__main__":
    main()
