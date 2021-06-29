"""
IS_LOCAL is bad, but we can use it to guide programmers.
"""
import os

from grapl_common.utils.primitive_convertors import to_bool

# Hey, don't consume this - instead consume the two assert methods below.
_IS_LOCAL = to_bool(os.getenv("IS_LOCAL", default=False))


def assert_not_local(msg: str) -> None:
    assert not _IS_LOCAL, msg


def assert_local(msg: str) -> None:
    assert _IS_LOCAL, msg
