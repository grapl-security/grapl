# (generated with --quick)

from typing import Any, List, Optional, Tuple, Type, TypeVar, Union

IntEq = Eq

Cmp: Type[Union[Contains, Distance, EndsWith, Eq, Has, Rex, StartsWith]]
IntCmp: Type[Union[Eq, Ge, Gt, Has, Le, Lt]]
IntOrNot: Type[Union[Not, int]]
OneOrMany: Any
StrCmp: Type[Union[Contains, Distance, EndsWith, Eq, Has, Rex, StartsWith]]
StrOrNot: Type[Union[Not, str]]
re: module

T = TypeVar("T")

class Contains:
    negated: bool
    predicate: str
    value: str
    def __init__(self, predicate: str, value: Union[Not, str]) -> None: ...
    def to_filter(self) -> str: ...

class Distance:
    distance: int
    negated: bool
    predicate: str
    value: Any
    def __init__(
        self, predicate: str, value: Union[Not, str], distance: int
    ) -> None: ...
    def to_filter(self) -> str: ...

class EndsWith:
    negated: bool
    predicate: str
    value: str
    def __init__(self, predicate: str, value: Union[Not, str]) -> None: ...
    def to_filter(self) -> str: ...

class Eq:
    negated: bool
    predicate: str
    value: Any
    def __init__(self, predicate: str, value: Union[Not, int, str]) -> None: ...
    def to_filter(self) -> str: ...

class Ge:
    negated: bool
    predicate: str
    value: int
    def __init__(self, predicate: str, value: Union[Not, int]) -> None: ...
    def to_filter(self) -> str: ...

class Gt:
    negated: bool
    predicate: str
    value: int
    def __init__(self, predicate: str, value: Union[Not, int]) -> None: ...
    def to_filter(self) -> str: ...

class Has:
    negated: bool
    predicate: Any
    def __init__(self, predicate: Union[Not, str]) -> None: ...
    def to_filter(self) -> str: ...

class Le:
    negated: bool
    predicate: str
    value: int
    def __init__(self, predicate: str, value: Union[Not, int]) -> None: ...
    def to_filter(self) -> str: ...

class Lt:
    negated: bool
    predicate: str
    value: int
    def __init__(self, predicate: str, value: Union[Not, int]) -> None: ...
    def to_filter(self) -> str: ...

class Not:
    value: Union[int, str]
    def __init__(self, value: Union[int, str]) -> None: ...

class Rex:
    negated: bool
    predicate: str
    value: Any
    def __init__(self, predicate: str, value: Union[Not, str]) -> None: ...
    def to_filter(self) -> str: ...

class StartsWith:
    negated: bool
    predicate: str
    value: str
    def __init__(self, predicate: str, value: Union[Not, str]) -> None: ...
    def to_filter(self) -> str: ...

def _int_cmps(
    predicate: str,
    eq: Optional[Union[Not, int]] = ...,
    gt: Optional[Union[Not, int]] = ...,
    ge: Optional[Union[Not, int]] = ...,
    lt: Optional[Union[Not, int]] = ...,
    le: Optional[Union[Not, int]] = ...,
) -> List[List[Union[Contains, Distance, EndsWith, Eq, Has, Rex, StartsWith]]]: ...
def _str_cmps(
    predicate: str,
    eq: Optional[Union[Not, str]] = ...,
    contains=...,
    ends_with: Optional[Union[Not, str]] = ...,
    starts_with: Optional[Union[Not, str]] = ...,
    regexp=...,
    distance_lt: Optional[Tuple[Union[Not, str], int]] = ...,
) -> List[List[Union[Contains, Distance, EndsWith, Eq, Has, Rex, StartsWith]]]: ...
def dgraph_prop_type(
    cmp: Union[Contains, Distance, EndsWith, Eq, Has, Rex, StartsWith]
) -> str: ...
def escape_dgraph_regexp(input: str) -> str: ...
def extract_value(value: Union[Not, int, str]) -> Union[int, str]: ...
