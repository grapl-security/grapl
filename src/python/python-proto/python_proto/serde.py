from __future__ import annotations

from abc import ABCMeta, abstractmethod
from typing import ClassVar, Generic, TypeVar

from google.protobuf.message import Message as _Message

P = TypeVar("P", bound=_Message)
T = TypeVar("T", bound="SerDe")
TInner = TypeVar("TInner", bound="SerDeWithInner")


class SerDe(Generic[P], metaclass=ABCMeta):
    proto_cls: ClassVar[type[P]]

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

    @classmethod
    def deserialize(cls: type[T], bytes_: bytes) -> T:
        proto_value = cls.proto_cls()
        proto_value.ParseFromString(bytes_)
        return cls.from_proto(proto_value)

    def serialize(self) -> bytes:
        return self.into_proto().SerializeToString()

    @classmethod
    @abstractmethod
    def from_proto(cls: type[T], proto: P) -> T:
        raise NotImplementedError

    @abstractmethod
    def into_proto(self) -> P:
        raise NotImplementedError


I = TypeVar("I", bound=SerDe)


class SerDeWithInner(Generic[I, P]):
    proto_cls: ClassVar[type[P]]
    inner_message: I

    @classmethod
    def __subclasshook__(cls, subclass: SerDeWithInner[I, P]) -> bool:
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

    @classmethod
    def deserialize(cls: type[TInner], bytes_: bytes, inner_cls: type[I]) -> TInner:
        proto_value = cls.proto_cls()
        proto_value.ParseFromString(bytes_)
        return cls.from_proto(proto_value, inner_cls)

    def serialize(self) -> bytes:
        return self.into_proto().SerializeToString()

    @classmethod
    @abstractmethod
    def from_proto(cls: type[TInner], proto: P, inner_cls: type[I]) -> TInner:
        raise NotImplementedError

    @abstractmethod
    def into_proto(self) -> P:
        raise NotImplementedError
