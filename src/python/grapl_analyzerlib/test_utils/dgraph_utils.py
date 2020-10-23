import json
import unittest
from typing import Dict, Type, Any

from grapl_analyzerlib.node_types import PropType
from grapl_analyzerlib.viewable import Viewable
from grapl_analyzerlib.dgraph_mutate import upsert
from grapl_analyzerlib.grapl_client import GraphClient


def create_edge(
    client: GraphClient, from_uid: str, edge_name: str, to_uid: str
) -> None:
    if edge_name[0] == "~":
        mut = {"uid": to_uid, edge_name[1:]: {"uid": from_uid}}

    else:
        mut = {"uid": from_uid, edge_name: {"uid": to_uid}}

    with client.txn_context(read_only=False) as txn:
        txn.mutate(set_obj=mut, commit_now=True)


def node_key_for_test(test_case: unittest.TestCase, node_key: str) -> str:
    """
    The atrociously-named TestCase#id returns things like
    tests.test_ip_address_node.TestIpAddressQuery.test__single_ip_addr_node__query_by_node_key
    """
    return "{}{}".format(test_case.id(), node_key)
