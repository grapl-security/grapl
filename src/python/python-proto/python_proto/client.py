from __future__ import annotations

from dataclasses import dataclass
from typing import Generic, Protocol, TypeVar

T = TypeVar("T", covariant=True)


@dataclass(frozen=True, slots=True)
class GrpcClientConfig:
    address: str

    def __post_init__(self) -> None:
        if not self.address.startswith("http://"):
            raise ValueError(f"Expected {self.address} to start with http://")


class Connectable(Protocol, Generic[T]):
    @classmethod
    def connect(cls: type[T], client_config: GrpcClientConfig) -> T:
        ...
