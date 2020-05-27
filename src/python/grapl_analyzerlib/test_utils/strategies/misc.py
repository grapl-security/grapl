from typing import Callable, Text, TypeVar

from hypothesis import assume
from hypothesis import strategies as st


T = TypeVar("T", bound=str)


@st.composite
def text_dgraph_compat(draw: Callable[[st.SearchStrategy[str]], str],) -> str:
    base_text = draw(st.text(min_size=3))
    # Don't fuck with newlines due to a dgraph bug
    # https://github.com/dgraph-io/dgraph/issues/4694
    assume(len(base_text) > 3)
    assume("\n" not in base_text)
    assume("\\" not in base_text)
    return base_text
