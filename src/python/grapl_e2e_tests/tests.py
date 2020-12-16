import logging
from unittest import TestCase

from grapl_tests_common.wait import WaitForCondition, WaitForQuery, wait_for_one

from grapl_analyzerlib.nodes.lens import LensQuery, LensView
from grapl_analyzerlib.retry import retry

LENS_NAME = "DESKTOP-FVSHABR"


class TestEndToEnd(TestCase):
    def test_expected_data_in_dgraph(self) -> None:
        query = LensQuery().with_lens_name(LENS_NAME)
        lens: LensView = wait_for_one(WaitForQuery(query), timeout_secs=120)
        assert lens.get_lens_name() == LENS_NAME

        # lens scope is not atomic
        def condition() -> bool:
            length = len(lens.get_scope())
            logging.info(f"Expected 4 nodes in scope, currently is {length}")
            return length == 4

        wait_for_one(WaitForCondition(condition), timeout_secs=240)
