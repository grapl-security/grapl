from abc import ABCMeta, abstractmethod, abstractstaticmethod
from typing import Any

class SerDe(metaclass=ABCMeta):

    @classmethod
    def __subclasshook__(cls, subclass: Any):
        return (
            hasattr(subclass, "deserialize") and
            callable(subclass.deserialize) and
            hasattr(subclass, "serialize") and
            callable(subclass.serialize) and
            hasattr(subclass, "_from_proto") and
            callable(subclass._from_proto) and
            hasattr(subclass, "_into_proto") and
            callable(subclass._into_proto)
        )

    @abstractstaticmethod
    def deserialize(bytes_: bytes) -> SerDe:
        raise NotImplementedError

    @abstractmethod
    def serialize(self) -> bytes:
        raise NotImplementedError

    @abstractstaticmethod
    def _from_proto(proto: Any) -> SerDe:
        raise NotImplementedError

    @abstractmethod
    def _into_proto(self) -> Any:
        raise NotImplementedError
