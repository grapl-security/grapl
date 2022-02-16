from typing import Type, Union, cast

import hypothesis.strategies as st
from hypothesis import given
from python_proto import I, P, SerDe, SerDeWithInner


def check_encode_decode_invariant(
    strategy: st.SearchStrategy[Union[SerDe[P], SerDeWithInner[P, I]]]
) -> None:
    @given(strategy)
    def _check_encode_decode_invariant(
        serde_value: Union[SerDe[P], SerDeWithInner[P, I]]
    ) -> None:
        if isinstance(serde_value, SerDeWithInner):
            transformed_with_inner = cast(
                SerDeWithInner[P, I],
                serde_value.__class__.deserialize(
                    serde_value.serialize(),
                    inner_cls=cast(
                        Type, serde_value.inner_message.__class__
                    ),  # lol what
                ),
            )
            assert transformed_with_inner == serde_value
        else:
            transformed = cast(
                SerDe[P], serde_value.__class__.deserialize(serde_value.serialize())
            )
            assert transformed == serde_value

    _check_encode_decode_invariant()
