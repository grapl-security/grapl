from typing import Callable

from hypothesis import assume
from hypothesis import strategies as st


@st.composite
def text_dgraph_compat(
    draw: Callable[[st.SearchStrategy[str]], str], max_size: int = 64
) -> str:
    base_text = draw(st.text(min_size=4, max_size=max_size))
    # Don't fuck with newlines due to a dgraph bug
    # https://github.com/dgraph-io/dgraph/issues/4694
    assume(len(base_text) > 3)
    assume("\n" not in base_text)
    assume("\\" not in base_text)
    assume("$" not in base_text)  # FIXME: this is probably due to a DGraph bug
    return base_text
