from typing import TypeVar
from grapl_analyzerlib.nodes.viewable import Viewable

V = TypeVar("V", bound=Viewable)


def assert_views_equal(*, expected: V, actual: V):
    assert expected.get_properties() == actual.get_properties()
