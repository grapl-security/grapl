from __future__ import annotations
import uuid
import json
import unittest
from typing import Dict, Any, Type

from grapl_analyzerlib.viewable import Viewable
from grapl_analyzerlib.queryable import V
from grapl_analyzerlib.grapl_client import GraphClient


def _upsert(client: GraphClient, node_dict: Dict[str, Any]) -> int:
    node_dict["uid"] = "_:blank-0"
    node_key = node_dict["node_key"]
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}"), first: 1) {{
                    uid,
                    dgraph.type
                    expand(_all_)
            }}
        }}
        """

    with client.txn_context(read_only=False) as txn:
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
        return int(new_uid, 16)


def upsert(
    client: GraphClient,
    type_name: str,
    view_type: Type[V],
    node_key: str,
    node_props: Dict[str, Any],
) -> V:
    node_props["node_key"] = node_key
    node_props["dgraph.type"] = list({type_name, "Base", "Entity"})
    uid = _upsert(client, node_props)
    node_props["uid"] = uid
    return view_type.from_dict(node_props, client)


def create_edge(
    client: GraphClient, from_uid: int, edge_name: str, to_uid: int
) -> None:
    if edge_name[0] == "~":
        mut = {"uid": to_uid, edge_name[1:]: {"uid": from_uid}}

    else:
        mut = {"uid": from_uid, edge_name: {"uid": to_uid}}

    with client.txn_context(read_only=False) as txn:
        txn.mutate(set_obj=mut, commit_now=True)


def random_key_for_test(test_case: unittest.TestCase) -> str:
    """
    The atrociously-named TestCase#id returns things like
    tests.test_ip_address_node.TestIpAddressQuery.test__single_ip_addr_node__query_by_node_key
    letting us tie back a node to the test that created it
    """
    return "{}{}".format(test_case.id(), uuid.uuid4())
