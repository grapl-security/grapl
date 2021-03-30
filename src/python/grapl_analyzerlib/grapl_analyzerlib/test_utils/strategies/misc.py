from typing import Callable, Text, TypeVar

from hypothesis import assume
from hypothesis import strategies as st
import unittest
from random import Random
from uuid import UUID


T = TypeVar("T", bound=str)


@st.composite
def text_dgraph_compat(
    draw: Callable[[st.SearchStrategy[str]], str], max_size: int = 128
) -> str:
    base_text = draw(st.text(min_size=3, max_size=max_size))
    # Don't fuck with newlines due to a dgraph bug
    # https://github.com/dgraph-io/dgraph/issues/4694
    assume(len(base_text) > 3)
    assume("\n" not in base_text)
    assume("\\" not in base_text)
    assume("$" not in base_text)  # FIXME: this is probably due to a DGraph bug
    return base_text


def build_random_key() -> st.SearchStrategy[str]:
    def get_test_id(runner: unittest.TestCase, random: Random):
        """
        Generates a string that will:
        - Identify which test created a node
        - have some randomness to avoid key collisions

        For example, it might look like this minus the \n's:
        tests.test_process_node.TestProcessQuery.test_single_process_connected_to_asset_node
        -
        some-uuid-goes-here-01234
        """
        test_id = runner.id()
        rand_bits = random.getrandbits(128)
        print(f"random bits: {rand_bits}")
        random_suffix = UUID(int=rand_bits, version=4)
        string = f"{test_id}-{random_suffix}"
        return string

    return st.builds(
        get_test_id, runner=st.runner(), random=st.randoms(use_true_random=True)
    )
