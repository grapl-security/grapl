import unittest
import pytest
from grapl_analyzerlib.comparators import (
    Distance,
    Not,
    Has,
    Eq,
)
from typing_extensions import Final

PREDICATE: Final[str] = "pred"
VALUE: Final[str] = "value"


class TestComparators(unittest.TestCase):
    def test_distance(self) -> None:
        comparator = Distance(
            predicate=PREDICATE,
            value=VALUE,
            distance=3,
        )
        assert comparator.to_filter() == "distance(pred, value, 3)"

        comparator = Distance(
            predicate=PREDICATE,
            value=Not(VALUE),
            distance=3,
        )
        assert comparator.to_filter() == "NOT distance(pred, value, 3)"

    def test_has(self) -> None:
        comparator = Has(
            predicate=PREDICATE,
        )
        assert comparator.to_filter() == "has(pred)"

        comparator = Has(
            predicate=Not(PREDICATE),
        )
        assert comparator.to_filter() == "(NOT has(pred) )"

    def test_eq__non_dgraph_type(self) -> None:
        comparator = Eq(
            predicate=PREDICATE,
            value=VALUE,
        )
        assert comparator.to_filter() == "eq(pred, value)"

        comparator = Eq(
            predicate=PREDICATE,
            value=Not(VALUE),
        )
        assert comparator.to_filter() == "(NOT eq(pred, value))"

    def test_eq__dgraph_type(self) -> None:
        comparator = Eq(
            predicate="dgraph.type",
            value=VALUE,
        )
        assert comparator.to_filter() == "type(value)"

        comparator = Eq(
            predicate="dgraph.type",
            value=Not(VALUE),
        )
        assert comparator.to_filter() == "(NOT type(value))"

    @pytest.mark.skip("TODO")
    def test_gt(self) -> None:
        pass

    @pytest.mark.skip("TODO")
    def test_ge(self) -> None:
        pass

    @pytest.mark.skip("TODO")
    def test_lt(self) -> None:
        pass

    @pytest.mark.skip("TODO")
    def test_le(self) -> None:
        pass

    @pytest.mark.skip("TODO")
    def test_contains(self) -> None:
        pass

    @pytest.mark.skip("TODO")
    def test_startswith(self) -> None:
        pass

    @pytest.mark.skip("TODO")
    def test_endswith(self) -> None:
        pass

    @pytest.mark.skip("TODO")
    def test_rex(self) -> None:
        pass
