import re
from typing import Any, List, Union, TypeVar, Optional, Tuple


T = TypeVar("T")
OneOrMany = Union[T, List[T]]


def escape_dgraph_regexp(input: str) -> str:
    input = re.escape(input)
    output = ""
    for char in input:
        if char == '"':
            output += r"\""
        elif char == "/":
            output += r"\/"
        else:
            output += char

    return output


class Not(object):
    def __init__(self, value: Union[str, int]):
        self.value = value


StrOrNot = Union[str, Not]
IntOrNot = Union[int, Not]


class Has(object):
    def __init__(self, predicate: StrOrNot):
        self.predicate = extract_value(predicate)
        self.negated = isinstance(predicate, Not)

    def to_filter(self) -> str:
        filter_str = "has({})".format(
            self.predicate,
        )
        if self.negated:
            filter_str = f"(NOT {filter_str} )"
        return filter_str


class Eq(object):
    def __init__(self, predicate: str, value: Union[Not, str, int]):
        self.predicate = predicate
        self.value = extract_value(value)
        self.negated: bool = isinstance(value, Not)

    def to_filter(self) -> str:
        if self.predicate == "dgraph.type":
            filter_str = f"type({self.value})"
        else:
            filter_str = f"eq({self.predicate}, {self.value})"

        if self.negated:
            return f"(NOT {filter_str})"
        else:
            return filter_str


class Gt(object):
    def __init__(self, predicate: str, value: IntOrNot):
        self.predicate = predicate
        self.value = int(extract_value(value))
        self.negated: bool = isinstance(value, Not)

    def to_filter(self) -> str:
        filter_str = "gt({}, {})".format(
            self.predicate,
            self.value,
        )

        if self.negated:
            return "(NOT " + filter_str + ")"
        else:
            return filter_str


class Ge(object):
    def __init__(self, predicate: str, value: IntOrNot):
        self.predicate = predicate
        self.value = int(extract_value(value))
        self.negated: bool = isinstance(value, Not)

    def to_filter(self) -> str:
        filter_str = "ge({}, {})".format(
            self.predicate,
            self.value,
        )

        if self.negated:
            return "(NOT " + filter_str + ")"
        else:
            return filter_str


class Lt(object):
    def __init__(self, predicate: str, value: IntOrNot):
        self.predicate = predicate
        self.value = int(extract_value(value))
        self.negated: bool = isinstance(value, Not)

    def to_filter(self) -> str:
        filter_str = "lt({}, {})".format(
            self.predicate,
            self.value,
        )

        if self.negated:
            return "(NOT " + filter_str + ")"
        else:
            return filter_str


class Le(object):
    def __init__(self, predicate: str, value: IntOrNot):
        self.predicate = predicate
        self.value = int(extract_value(value))
        self.negated: bool = isinstance(value, Not)

    def to_filter(self) -> str:
        filter_str = "le({}, {})".format(
            self.predicate,
            self.value,
        )

        if self.negated:
            return "(NOT " + filter_str + ")"
        else:
            return filter_str


class Contains(object):
    def __init__(self, predicate: str, value: StrOrNot):
        self.predicate = predicate
        self.value = re.escape(str(extract_value(value)))
        self.negated: bool = isinstance(value, Not)

    def to_filter(self) -> str:
        if self.negated:
            return f"NOT regexp({self.predicate}, /.*{self.value}.*/i)"
        else:
            return f"regexp({self.predicate}, /.*{self.value}.*/i)"


IntEq = Eq


class StartsWith(object):
    def __init__(self, predicate: str, value: StrOrNot):
        self.predicate = predicate
        self.value = re.escape(str(extract_value(value)))
        self.negated: bool = isinstance(value, Not)

    def to_filter(self) -> str:
        if self.negated:
            return f"NOT regexp({self.predicate}, /^{self.value}/)"
        else:
            return f"regexp({self.predicate}, /^{self.value}/)"


class EndsWith(object):
    def __init__(self, predicate: str, value: StrOrNot):
        self.predicate = predicate
        self.value = re.escape(str(extract_value(value)))
        self.negated = isinstance(value, Not)

    def to_filter(self) -> str:
        if self.negated:
            return f"NOT regexp({self.predicate}, /{self.value}$/)"
        else:
            return f"regexp({self.predicate}, /{self.value}$/)"


class Rex(object):
    def __init__(self, predicate: str, value: StrOrNot):
        self.predicate = predicate
        self.value = extract_value(value)
        self.negated: bool = isinstance(value, Not)

    def to_filter(self) -> str:
        if self.negated:
            return f"NOT regexp({self.predicate}, /{self.value}/)"
        else:
            return f"regexp({self.predicate}, /{self.value}/)"


class Distance(object):
    def __init__(self, predicate: str, value: StrOrNot, distance: int):
        self.predicate = predicate
        self.value = extract_value(value)
        self.negated: bool = isinstance(value, Not)
        self.distance = distance

    def to_filter(self) -> str:
        distance_query = f"distance({self.predicate}, {self.value}, {self.distance})"
        if self.negated:
            return f"NOT {distance_query}"
        else:
            return distance_query


StrCmp = Union[Has, Eq, Contains, StartsWith, EndsWith, Rex, Distance]
IntCmp = Union[Has, Eq, Gt, Lt, Ge, Le]
Cmp = Union[StrCmp]


def dgraph_prop_type(cmp: Cmp) -> str:
    if isinstance(cmp, Has):
        return "string"
    if isinstance(cmp, Eq):
        if isinstance(cmp.value, str):
            return "string"
        else:
            return "int"
    if isinstance(cmp, Contains):
        return "string"
    if isinstance(cmp, StartsWith):
        return "string"
    if isinstance(cmp, EndsWith):
        return "string"
    if isinstance(cmp, Rex):
        return "string"
    if isinstance(cmp, Distance):
        return "string"

    return "int"


def _str_cmps(
    predicate: str,
    eq: Optional[StrOrNot] = None,
    contains: Optional[OneOrMany[StrOrNot]] = None,
    ends_with: Optional[StrOrNot] = None,
    starts_with: Optional[StrOrNot] = None,
    regexp: Optional[OneOrMany[StrOrNot]] = None,
    distance_lt: Optional[Tuple[StrOrNot, int]] = None,
) -> List[List[StrCmp]]:
    cmps = []  # type: List[List[Any]]

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

    if isinstance(starts_with, str) or isinstance(starts_with, Not):
        cmps.append([StartsWith(predicate, starts_with)])

    if isinstance(regexp, str) or isinstance(regexp, Not):
        cmps.append([Rex(predicate, regexp)])
    elif isinstance(regexp, list):
        _regexp = [Rex(predicate, e) for e in regexp]
        cmps.append(_regexp)

    if distance_lt:
        cmps.append([Distance(predicate, distance_lt[0], distance_lt[1])])

    if not cmps:
        cmps.append([Has(predicate)])

    return cmps


def _int_cmps(
    predicate: str,
    eq: Optional[IntOrNot] = None,
    gt: Optional[IntOrNot] = None,
    ge: Optional[IntOrNot] = None,
    lt: Optional[IntOrNot] = None,
    le: Optional[IntOrNot] = None,
) -> List[List[StrCmp]]:
    cmps = []  # type: List[List[Any]]

    if isinstance(eq, str) or isinstance(eq, Not):
        cmps.append([Eq(predicate, eq)])

    if isinstance(gt, str) or isinstance(gt, Not):
        cmps.append([Eq(predicate, gt)])

    if isinstance(ge, str) or isinstance(ge, Not):
        cmps.append([Eq(predicate, ge)])

    if isinstance(lt, str) or isinstance(lt, Not):
        cmps.append([Eq(predicate, lt)])

    if isinstance(le, str) or isinstance(le, Not):
        cmps.append([Eq(predicate, le)])

    if not cmps:
        cmps.append([Has(predicate)])

    return cmps


def extract_value(value: Union[Not, int, str]) -> Union[int, str]:
    if isinstance(value, Not):
        return value.value
    else:
        return value
