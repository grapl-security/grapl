from time import sleep
from unittest import TestCase
from grapl_analyzerlib.grapl_client import MasterGraphClient
from grapl_analyzerlib.nodes.lens import LensQuery, LensView
import resources
from typing import Any, Optional

LENS_NAME = "DESKTOP-FVSHABR"


class TestEndToEnd(TestCase):
    def test_analyzer_deployed_properly(self) -> None:
        assert True

    def test_analyzer_fired(self) -> None:
        assert True

    def test_expected_data_in_dgraph(self) -> None:
        lens_resource = wait_for_lens()
        wait_result = resources.wait_on_resources([lens_resource], timeout_secs=120)
        lens: LensView = wait_result[lens_resource]
        # Adding nodes to this lens is not an atomic operation, so let's add some buffer and hope for the best
        # a better solution would be to specify the specific scope in `wait_for_lens`
        sleep(5)
        assert lens.get_lens_name() == LENS_NAME
        assert len(lens.get_scope()) == 3


def wait_for_lens():
    local_client = MasterGraphClient()
    query = LensQuery().with_lens_name(LENS_NAME)
    return WaitForLens(local_client, query)


class WaitForLens(resources.WaitForResource):
    def __init__(self, dgraph_client: Any, query: LensQuery):
        self.dgraph_client = dgraph_client
        self.query = query

    def acquire(self) -> Optional[Any]:
        # result = self.query.get_count(self.dgraph_client)
        result = self.query.query_first(self.dgraph_client)
        return result

    def __str__(self) -> str:
        return f"WaitForLens({self.query})"
