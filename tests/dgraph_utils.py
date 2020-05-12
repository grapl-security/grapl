import json
import unittest
from typing import Dict, Type

from pydgraph import DgraphClient
from grapl_analyzerlib.nodes.types import Property
from grapl_analyzerlib.nodes.viewable import Viewable
from grapl_analyzerlib.nodes.comparators import escape_dgraph_str


def _upsert(client: DgraphClient, node_dict: Dict[str, Property]) -> str:
    if node_dict.get("uid"):
        node_dict.pop("uid")
    node_dict["uid"] = "_:blank-0"
    node_key = node_dict["node_key"]
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}"))
            {{
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
    for key, value in node_props.items():
        if isinstance(value, str):
            node_props[key] = escape_dgraph_str(value)
    uid = _upsert(client, node_props)
    # print(f'uid: {uid}')
    node_props["uid"] = uid
    # print(node_props['node_key'])
    return view_type.from_dict(client, node_props)


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


def test_name_plus_node_key(test_case: unittest.TestCase, node_key: str) -> str:
    """
    The atrociously-named TestCase#id returns things like
    tests.test_ip_address_node.TestIpAddressQuery.test__single_ip_addr_node__query_by_node_key
    """
    return "{}{}".format(test_case.id(), node_key)
