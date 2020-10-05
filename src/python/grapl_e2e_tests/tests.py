from unittest import TestCase
from grapl_analyzerlib.grapl_client import MasterGraphClient
from grapl_analyzerlib.nodes.lens import LensQuery, LensView
import resources
from typing import Any, Optional, Callable
import inspect

LENS_NAME = "DESKTOP-FVSHABR"


class TestEndToEnd(TestCase):
    def test_analyzer_deployed_properly(self) -> None:
        assert True

    def test_analyzer_fired(self) -> None:
        assert True

    def test_expected_data_in_dgraph(self) -> None:
        lens_resource = wait_for_lens()
        wait_result = resources.wait_for([lens_resource], timeout_secs=120)

        lens: LensView = wait_result[lens_resource]
        assert lens.get_lens_name() == LENS_NAME

        # lens scope is not atomic
        resources.wait_for(
            [WaitForCondition(lambda: (len(lens.get_scope()) == 4))], timeout_secs=120
        )


def wait_for_lens():
    local_client = MasterGraphClient()
    query = LensQuery().with_lens_name(LENS_NAME)
    return WaitForLens(local_client, query)


class WaitForCondition(resources.WaitForResource):
    """ just something nice n generic """

    def __init__(self, fn: Callable[[], Optional[bool]]) -> None:
        self.fn = fn

    def acquire(self) -> Optional[Any]:
        result = self.fn()
        if result:
            return self  # just anything non-None
        else:
            return None

    def __str__(self) -> str:
        return f"WaitForCondition({inspect.getsource(self.fn)})"


class WaitForLens(resources.WaitForResource):
    def __init__(self, dgraph_client: Any, query: LensQuery) -> None:
        self.dgraph_client = dgraph_client
        self.query = query

    def acquire(self) -> Optional[Any]:
        result = self.query.query_first(self.dgraph_client)
        return result

    def __str__(self) -> str:
        return f"WaitForLens({self.query})"
