import pytest

pytest.register_assert_rewrite("python_proto.tests.helpers")

import hypothesis.strategies as st
from python_proto.grapl.common.v1beta1 import messages
from python_proto.tests import strategies
from python_proto.tests.helpers import check_encode_decode_invariant


def uids(
    value: st.SearchStrategy[int] = st.integers(
        min_value=strategies.UINT64_MIN, max_value=strategies.UINT64_MAX
    ),
) -> st.SearchStrategy[messages.Uid]:
    return st.builds(messages.Uid, value=value)


def property_names(
    value: st.SearchStrategy[str] = st.text(),
) -> st.SearchStrategy[messages.PropertyName]:
    return st.builds(messages.PropertyName, value=value)


def test_uids() -> None:
    check_encode_decode_invariant(uids())


def test_property_names() -> None:
    check_encode_decode_invariant(property_names())
