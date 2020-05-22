import unittest
import inspect
from typing import Optional, Dict, cast

import hypothesis
import hypothesis.strategies as st
from hypothesis import given
from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzerlib.nodes.ip_address_node import IpAddressView
from grapl_analyzerlib.nodes.ip_address_node import IpAddressQuery
from grapl_analyzerlib.nodes.types import Property
from grapl_analyzerlib.time_utils import Millis, as_millis

from test_utils.dgraph_utils import upsert, node_key_for_test
from test_utils.view_assertions import assert_views_equal


def get_or_create_ip_address_node(
    local_client: DgraphClient,
    node_key: str,
    first_seen_timestamp: Millis,
    last_seen_timestamp: Millis,
    ip_address: str,
    # ip connections todo
) -> IpAddressView:
    ip_addr_props = {
        "first_seen_timestamp": first_seen_timestamp,
        "last_seen_timestamp": last_seen_timestamp,
        "ip_address": ip_address,
        #'ip_connections': None
    }  # type: Dict[str, Property]

    return cast(
        IpAddressView,
        upsert(
            client=local_client,
            type_name="IpAddress",
            view_type=IpAddressView,
            node_key=node_key,
            node_props=ip_addr_props,
        ),
    )


class TestIpAddressQuery(unittest.TestCase):
    @hypothesis.settings(deadline=None,)
    @given(
        node_key=st.uuids(),
        first_seen_timestamp=st.datetimes(),
        last_seen_timestamp=st.datetimes(),
        ip_address=st.ip_addresses(v=4),  # TODO: support ipv6?
    )
    def test__single_ip_addr_node__query_by_node_key(
        self, node_key, first_seen_timestamp, last_seen_timestamp, ip_address,
    ):
        # current function's name, but don't need to copy-paste replace
        node_key = node_key_for_test(self, node_key)
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))

        created = get_or_create_ip_address_node(
            local_client=local_client,
            node_key=node_key,
            first_seen_timestamp=as_millis(first_seen_timestamp),
            last_seen_timestamp=as_millis(last_seen_timestamp),
            ip_address=str(ip_address),
        )

        queried_ip_address_node = (
            IpAddressQuery()
            .with_ip_address()
            .with_first_seen_timestamp()
            .with_last_seen_timestamp()
            .query_first(local_client, contains_node_key=node_key)
        )

        assert_views_equal(expected=created, actual=queried_ip_address_node)

    # TODO: Add tests around `ip_connections`


if __name__ == "__main__":
    unittest.main()
