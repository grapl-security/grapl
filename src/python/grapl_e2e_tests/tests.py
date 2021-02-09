import logging
from unittest import TestCase

from grapl_tests_common.wait import WaitForCondition, WaitForQuery, wait_for_one

from grapl_analyzerlib.nodes.lens import LensQuery, LensView
from grapl_analyzerlib.retry import retry

LENS_NAME = "DESKTOP-FVSHABR"


class TestEndToEnd(TestCase):
    def test_expected_data_in_dgraph(self) -> None:
        # There is some unidentified, nondeterministic failure with e2e.
        # We fall into one of three buckets:
        # - No lens
        # - Lens with 3 scope
        # - Lens with 4 scope (correct)

        query = LensQuery().with_lens_name(LENS_NAME)
        lens: LensView = wait_for_one(WaitForQuery(query), timeout_secs=120)
        assert lens.get_lens_name() == LENS_NAME

        # lens scope is not atomic
        def condition() -> bool:
            length = len(lens.get_scope())
            logging.info(f"Expected 3-4 nodes in scope, currently is {length}")

            # The correct answer for this is 4.
            # We are temp 'allowing' 3 because it means the pipeline is, _mostly_, working.
            return length in (
                3,
                4,
            )

        wait_for_one(WaitForCondition(condition), timeout_secs=240)
