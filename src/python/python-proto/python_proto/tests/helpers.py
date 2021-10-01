import hypothesis.strategies as st
from hypothesis import given
from python_proto import SerDe


def check_encode_decode_invariant(strategy: st.SearchStrategy[SerDe]) -> None:
    @given(strategy)
    def _check_encode_decode_invariant(serde_value: SerDe) -> None:
        transformed = serde_value.__class__.deserialize(serde_value.serialize())
        assert transformed == serde_value

    _check_encode_decode_invariant()
