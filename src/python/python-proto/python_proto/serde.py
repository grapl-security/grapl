from __future__ import annotations

from abc import ABCMeta, abstractmethod
from typing import ClassVar, Generic, TypeVar, cast

from google.protobuf.message import Message as _Message

P = TypeVar("P", bound=_Message)
T = TypeVar("T", bound="SerDe")
TInner = TypeVar("TInner", bound="SerDeWithInner")


class SerDe(Generic[P], metaclass=ABCMeta):
    _proto_cls: ClassVar[type]

    @classmethod
    def new_proto(cls) -> P:
        """
        An awful hack to get around the fact that you cannot have a TypeVar
        inside of a ClassVar (in this case, _proto_cls).
        Ideally we'd just have `proto_cls: ClassVar[type[P]]`
        https://github.com/python/mypy/issues/5144
        """
        return cast(P, cls._proto_cls())

    @classmethod
    def deserialize(cls: type[T], bytes_: bytes) -> T:
        proto_value = cls.new_proto()
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
    _proto_cls: ClassVar[type]
    inner_message: I

    @classmethod
    def new_proto(cls) -> P:
        """
        An awful hack to get around the fact that you cannot have a TypeVar
        inside of a ClassVar (in this case, _proto_cls).
        Ideally we'd just have `proto_cls: ClassVar[type[P]]`
        https://github.com/python/mypy/issues/5144
        """
        return cast(P, cls._proto_cls())

    @classmethod
    def deserialize(cls: type[TInner], bytes_: bytes, inner_cls: type[I]) -> TInner:
        proto_value = cls.new_proto()
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
