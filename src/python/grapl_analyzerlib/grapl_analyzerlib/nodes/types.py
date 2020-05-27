from typing import Type, Union, List, TypeVar


T = TypeVar("T")

OneOrMany = Union[T, List[T]]

PropertyT = Union[Type[str], Type[int]]
Property = Union[str, int]
