from __future__ import annotations

from dataclasses import dataclass
from typing import Generic, Protocol, TypeVar

T = TypeVar("T", covariant=True)


@dataclass(frozen=True, slots=True)
class GrpcClientConfig:
    # We don't yet have anything in the Client Config, but we may add
    # things like client-wide retry settings, etc.

    @classmethod
    def default(cls) -> GrpcClientConfig:
        return cls()


class Connectable(Protocol, Generic[T]):
    @classmethod
    def connect(cls: type[T], client_config: GrpcClientConfig) -> T:
        ...
