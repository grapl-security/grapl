from __future__ import annotations

from abc import ABCMeta, abstractmethod, abstractstaticmethod
from typing import Generic, Type, TypeVar

from google.protobuf.message import Message as _Message

P = TypeVar("P", bound=_Message)


class SerDe(Generic[P], metaclass=ABCMeta):
    proto_cls: Type[P]

    @classmethod
    def __subclasshook__(cls, subclass: SerDe[P]) -> bool:
        return (
            hasattr(subclass, "proto_cls")
            and hasattr(subclass, "deserialize")
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
    def deserialize(bytes_: bytes) -> SerDe[P]:
        raise NotImplementedError

    def serialize(self) -> bytes:
        return self.into_proto().SerializeToString()

    @staticmethod
    @abstractstaticmethod
    def from_proto(proto: P) -> SerDe[P]:
        raise NotImplementedError

    @abstractmethod
    def into_proto(self) -> P:
        raise NotImplementedError


I = TypeVar("I", bound=SerDe[_Message])


class SerDeWithInner(Generic[P, I]):
    proto_cls: Type[P]
    inner_message: I

    @classmethod
    def __subclasshook__(cls, subclass: SerDeWithInner[P, I]) -> bool:
        return (
            hasattr(subclass, "proto_cls")
            and hasattr(subclass, "inner_cls")
            and hasattr(subclass, "deserialize")
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
    def deserialize(bytes_: bytes, inner_cls: Type[I]) -> SerDeWithInner[P, I]:
        raise NotImplementedError

    def serialize(self) -> bytes:
        return self.into_proto().SerializeToString()

    @staticmethod
    @abstractstaticmethod
    def from_proto(proto: P, iner_cls: Type[I]) -> SerDeWithInner[P, I]:
        raise NotImplementedError

    @abstractmethod
    def into_proto(self) -> P:
        raise NotImplementedError
