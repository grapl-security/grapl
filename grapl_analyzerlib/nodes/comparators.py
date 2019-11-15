import re
from typing import List, Union, TypeVar, Generic, Optional, Sequence, Any, cast

from grapl_analyzerlib.nodes.types import PropertyT

T = TypeVar("T", bound=Union[str, int])

PropertyFilter = List[List["Cmp[T]"]]
StrCmp = Union[str, List[str], List[Union[str, 'Not[str]']]]
IntCmp = Union[int, List[int], List[Union[int, 'Not[int]']]]


class Or(object):
    def __init__(self, *values: PropertyT):
        self.values = values


class Not(Generic[T]):
    def __init__(self, value: T) -> None:
        self.value = value


class Cmp(Generic[T]):
    def to_filter(self) -> str:
        pass


class Eq(Cmp[T]):
    def __init__(
        self, predicate: str, value: Union[T, Not[T]]
    ) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, str):
            if self.predicate == "dgraph.type":
                return f'type({self.value})'
            return 'eq({}, "{}")'.format(self.predicate, self.value)
        if isinstance(self.value, int):
            return "eq({}, {})".format(self.predicate, self.value)
        if isinstance(self.value, Not) and isinstance(self.value.value, str):
            if self.predicate == "dgraph.type":
                return f'NOT type({self.value})'
            return 'NOT eq({}, "{}")'.format(self.predicate, self.value.value)
        if isinstance(self.value, Not) and isinstance(self.value.value, int):
            return "NOT eq({}, {})".format(self.predicate, self.value.value)
        raise TypeError


class EndsWith(Cmp[str]):
    def __init__(self, predicate: str, value: Union[str, Not[str]]) -> None:
        self.predicate = predicate
        self.value = value  # type: Union[str, Not[str]]

    def to_filter(self) -> str:
        if isinstance(self.value, Not):
            value = self.value.value
            escaped_value = re.escape(value)
            return "NOT regexp({}, /.*{}/i)".format(self.predicate, escaped_value)
        else:
            escaped_value = re.escape(self.value)
            return "regexp({}, /.*{}/i)".format(self.predicate, escaped_value)


class Rex(Cmp[str]):
    def __init__(self, predicate: str, value: Union[str, Not[str]]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, Not):
            value = self.value.value
            return f"NOT regexp({self.predicate}, /{value}/)"
        else:
            return f"regexp({self.predicate}, /{self.value}/)"


class Gt(Cmp[int]):
    def __init__(self, predicate: str, value: Union[int, Not[int]]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, Not):
            return f"NOT gt({self.predicate}, {self.value})"
        else:
            return f"gt({self.predicate}, {self.value})"


class Lt(Cmp[int]):
    def __init__(self, predicate: str, value: Union[int, Not[int]]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, Not):
            return f"NOT lt({self.predicate}, {self.value})"
        else:
            return f"lt({self.predicate}, {self.value})"


class Has(Cmp[Any]):
    def __init__(self, predicate: str) -> None:
        self.predicate = predicate

    def to_filter(self) -> str:
        return f"has({self.predicate})"


class Contains(Cmp[str]):
    def __init__(self, predicate: str, value: Union[str, Not[str]]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:

        if isinstance(self.value, Not):
            value = re.escape(self.value.value)
            return f"NOT regexp({self.predicate}, /{value}/)"
        else:
            value = re.escape(self.value)
            return f"regexp({self.predicate}, /{value}/)"


class Regexp(Cmp[str]):
    def __init__(self, predicate: str, value: Union[str, Not[str]]) -> None:
        self.predicate = predicate
        self.value = value

    def to_filter(self) -> str:

        if isinstance(self.value, Not):
            value = self.value.value.replace("/", "\\/")
            return f"NOT regexp({self.predicate}, /{value}/)"
        else:
            value = self.value.replace("/", "\\/")
            return f"regexp({self.predicate}, /{value}/)"


def _str_cmps(
        predicate: str,
        eq: Optional[StrCmp] = None,
        contains: Optional[StrCmp] = None,
        ends_with: Optional[StrCmp] = None,
) -> List[List[Cmp[str]]]:
    cmps = []  # type: List[Sequence[Cmp[str]]]

    if isinstance(eq, str) or isinstance(eq, Not):
        cmps.append([Eq(predicate, eq)])

    elif isinstance(eq, list):
        _eq = [Eq(predicate, e) for e in eq]
        cmps.append(_eq)

    if isinstance(contains, str) or isinstance(contains, Not):
        cmps.append([Contains(predicate, contains)])
    elif isinstance(contains, list):
        _contains = [Contains(predicate, e) for e in contains]
        cmps.append(_contains)

    if isinstance(ends_with, str) or isinstance(ends_with, Not):
        cmps.append([EndsWith(predicate, ends_with)])
    elif isinstance(ends_with, list):
        _ends_with = [EndsWith(predicate, e) for e in ends_with]
        cmps.append(_ends_with)

    if not (eq or contains or ends_with):
        cmps.append([Has(predicate)])

    return cast(List[List[Cmp[str]]], cmps)


def _int_cmps(
        predicate: str,
        eq: Optional[IntCmp] = None,
        gt: Optional[IntCmp] = None,
        lt: Optional[IntCmp] = None,
) -> List[List[Cmp[int]]]:
    cmps = []  # type: List[Sequence[Cmp[int]]]

    if isinstance(eq, int) or isinstance(eq, Not):
        cmps.append([Eq(predicate, eq)])
    elif isinstance(eq, list):
        _eq = [Eq(predicate, e) for e in eq]
        cmps.append(_eq)

    if isinstance(gt, int) or isinstance(gt, Not):
        cmps.append([Gt(predicate, gt)])
    elif isinstance(gt, list):
        _gt = [Gt(predicate, e) for e in gt]
        cmps.append(_gt)

    if isinstance(lt, int) or isinstance(lt, Not):
        cmps.append([Lt(predicate, lt)])
    elif isinstance(lt, list):
        _lt = [Lt(predicate, e) for e in lt]
        cmps.append(_lt)

    if not (eq or gt or lt):
        cmps.append([Has(predicate)])

    return cast(List[List[Cmp[int]]], cmps)
