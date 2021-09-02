from __future__ import annotations

from abc import ABCMeta, abstractmethod, abstractstaticmethod
from typing import Any


class SerDe(metaclass=ABCMeta):
    @classmethod
    def __subclasshook__(cls, subclass: Any) -> bool:
        return (
            hasattr(subclass, "deserialize")
            and callable(subclass.deserialize)
            and hasattr(subclass, "serialize")
            and callable(subclass.serialize)
            and hasattr(subclass, "_from_proto")
            and callable(subclass._from_proto)
            and hasattr(subclass, "_into_proto")
            and callable(subclass._into_proto)
        )

    @staticmethod
    @abstractstaticmethod
    def deserialize(bytes_: bytes) -> SerDe:
        raise NotImplementedError

    @abstractmethod
    def serialize(self) -> bytes:
        raise NotImplementedError

    @staticmethod
    @abstractstaticmethod
    def from_proto(proto: Any) -> SerDe:
        raise NotImplementedError

    @abstractmethod
    def into_proto(self) -> Any:
        raise NotImplementedError
