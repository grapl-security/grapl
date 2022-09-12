import hypothesis.strategies as st
from hypothesis import given
from python_proto.serde import SerDe, SerDeWithInner, T, TInner


def check_encode_decode_invariant(strategy: st.SearchStrategy[T | TInner]) -> None:
    @given(strategy)
    def _check_encode_decode_invariant(serde_value: T | TInner) -> None:
        if isinstance(serde_value, SerDeWithInner):
            transformed_with_inner = serde_value.__class__.deserialize(
                serde_value.serialize(),
                inner_cls=serde_value.inner_message.__class__,
            )
            assert transformed_with_inner == serde_value
        elif isinstance(serde_value, SerDe):
            transformed = serde_value.__class__.deserialize(serde_value.serialize())
            assert transformed == serde_value
        else:
            assert False, f"Unknown type {serde_value}"

    _check_encode_decode_invariant()
