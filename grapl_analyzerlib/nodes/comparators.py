
import re
from typing import List, Union, TypeVar, Generic, Optional, Sequence, Any, cast, Tuple

from grapl_analyzerlib.nodes.types import PropertyT

T = TypeVar("T", bound=Union[str, int])

PropertyFilter = List[List["Cmp[T]"]]
StrCmp = Union[str, List[str], List[Union[str, "Not[str]"]]]
IntCmp = Union[int, List[int], List[Union[int, "Not[int]"]]]

def escape_dgraph_regexp(input: str) -> str:
    input = re.escape(input)
    input = input.replace("\\$", "//\\$")
    output = ""
    for char in input:
        if char == '"':
            output += r'\"'
        elif char == '/':
            output += r'\/'
        else:
            output += char

    return output


def escape_dgraph_str(input: str, query=False) -> str:

    output = ""
    for char in input:
        if char == "$":
            output += "//$"
        elif char == "\n":
            if query:
                output += r"//\\n"
            else:
                output += r"//\n"
        elif char == "\\":
            if query:
                output += r"\\\\"
            else:
                output += r"\\"
        elif char == '"':
            if query:
                output += r'\"'
            else:
                output += r'"'
        else:
            output += char

    return output


def unescape_dgraph_str(input: str) -> str:
    if not isinstance(input, str):
        return input
    output = input.replace("//$", "$")
    output = output.replace(r"//\n", "\n")
    output = output.replace(r'\"', '"')
    output = output.replace(r'\\', '\\')
    return output



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
    def __init__(self, predicate: str, value: Union[T, Not[T]]) -> None:
        self.predicate = predicate
        if isinstance(value, str):
            self.value = escape_dgraph_str(value, query=True)  # type: Union[str, Not[str]]
        elif isinstance(value, Not) and isinstance(value.value, str):
            self.value = escape_dgraph_str(value.value, query=True)
        else:
            self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, str):
            if self.predicate == "dgraph.type":
                return f"type({self.value})"
            return 'eq({}, "{}")'.format(
                self.predicate,
                self.value,
            )
        if isinstance(self.value, int):
            return 'eq({}, {})'.format(self.predicate, self.value)
        if isinstance(self.value, Not) and isinstance(self.value.value, str):
            if self.predicate == "dgraph.type":
                return f"NOT type({self.value})"
            return 'NOT eq({}, "{}")'.format(self.predicate, self.value.value)
        if isinstance(self.value, Not) and isinstance(self.value.value, int):
            return "NOT eq({}, {})".format(self.predicate, self.value.value)
        raise TypeError


class EndsWith(Cmp[str]):
    def __init__(self, predicate: str, value: Union[str, Not[str]]) -> None:
        self.predicate = predicate
        if isinstance(value, str):
            self.value = escape_dgraph_str(value)  # type: Union[str, Not[str]]
        else:
            value.value = escape_dgraph_str(value.value)
            self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, Not):
            value = self.value.value
            escaped_value = re.escape(value)
            return "NOT regexp({}, /{}$/i)".format(self.predicate, escaped_value)
        else:
            escaped_value = re.escape(self.value)
            return "regexp({}, /{}$/i)".format(self.predicate, escaped_value)


class StartsWith(Cmp[str]):
    def __init__(self, predicate: str, value: Union[str, Not[str]]) -> None:
        self.predicate = predicate
        if isinstance(value, str):
            self.value = escape_dgraph_str(value)  # type: Union[str, Not[str]]
        else:
            value.value = escape_dgraph_str(value.value)
            self.value = value

    def to_filter(self) -> str:
        if isinstance(self.value, Not):
            value = self.value.value
            escaped_value = re.escape(value)
            return "NOT regexp({}, /^{}.*/i)".format(self.predicate, escaped_value)
        else:
            escaped_value = re.escape(self.value)
            return "regexp({}, /^{}.*/i)".format(self.predicate, escaped_value)


class Rex(Cmp[str]):
    def __init__(self, predicate: str, value: Union[str, Not[str]]) -> None:
        self.predicate = predicate
        if isinstance(value, str):
            self.value = value.replace("$", "//$").replace("\n", "//\n")
        else:
            value.value.replace("$", "//$").replace("\n", "//\n")
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
        if isinstance(value, str):
            self.value = escape_dgraph_regexp(value)
        else:
            value.value = escape_dgraph_regexp(value.value)
            self.value = value

    def to_filter(self) -> str:

        if isinstance(self.value, Not):
            # value = re.escape(self.value.value)
            # value = value.replace("/", "\\/")

            return f"NOT regexp({self.predicate}, /{self.value.value}/)"
        else:
            # value = re.escape(self.value)
            # value = value
            # value = value.replace("/", "\\/")
            return f"regexp({self.predicate}, /{self.value}/)"


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


class Distance(Cmp[str]):
    def __init__(
        self, predicate: str, value: Union[str, Not[str]], distance: int
    ) -> None:
        self.predicate = predicate
        self.value = value
        self.distance = distance

    def to_filter(self) -> str:

        if isinstance(self.value, Not):
            value = self.value.value
            return f'NOT match({self.predicate}, "{value}", {self.distance})'
        else:
            value = self.value
            return f'match({self.predicate}, "{value}", {self.distance})'

def _str_cmps(
    predicate: str,
    eq: Optional[StrCmp] = None,
    contains: Optional[StrCmp] = None,
    ends_with: Optional[StrCmp] = None,
    starts_with: Optional[StrCmp] = None,
    regexp: Optional[StrCmp] = None,
    distance: Optional[Tuple[StrCmp, int]] = None,
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

    if isinstance(starts_with, str) or isinstance(starts_with, Not):
        cmps.append([StartsWith(predicate, starts_with)])
    elif isinstance(starts_with, list):
        _starts_with = [StartsWith(predicate, e) for e in starts_with]
        cmps.append(_starts_with)

    if isinstance(regexp, str) or isinstance(regexp, Not):
        cmps.append([Rex(predicate, regexp)])
    elif isinstance(regexp, list):
        _regexp = [Rex(predicate, e) for e in regexp]
        cmps.append(_regexp)

    if distance:
        if isinstance(distance[0], str) or isinstance(distance[0], Not):
            cmps.append([Distance(predicate, distance[0], distance[1])])
        elif isinstance(distance, list):
            _distance = [Distance(predicate, e[0], e[1]) for e in distance]
            cmps.append(_distance)

    if not cmps:
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

    if eq is None and gt is None and lt is None:
        cmps.append([Has(predicate)])

    return cast(List[List[Cmp[int]]], cmps)
