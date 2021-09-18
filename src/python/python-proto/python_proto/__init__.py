from __future__ import annotations

from abc import ABCMeta, abstractmethod, abstractstaticmethod
from typing import cast, Any


class SerDe(metaclass=ABCMeta):
    @classmethod
    def __subclasshook__(cls, subclass: SerDe) -> bool:
        return (
            hasattr(subclass, "deserialize")
            and callable(subclass.deserialize)
            and hasattr(subclass, "serialize")
            and callable(subclass.serialize)
            and hasattr(subclass, "from_proto")
            and callable(subclass.from_proto)
            and hasattr(subclass, "into_proto")
            and callable(subclass.into_proto)
        )

    @staticmethod
    @abstractstaticmethod
    def deserialize(bytes_: bytes) -> SerDe:
        raise NotImplementedError

    def serialize(self) -> bytes:
        return cast(bytes, self.into_proto().SerializeToString())

    @staticmethod
    @abstractstaticmethod
    def from_proto(proto: Any) -> SerDe:
        raise NotImplementedError

    @abstractmethod
    def into_proto(self) -> Any:
        raise NotImplementedError
