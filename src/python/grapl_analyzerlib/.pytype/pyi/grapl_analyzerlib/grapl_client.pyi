# (generated with --quick)

import contextlib
from typing import Any, Callable, Iterator, Tuple, TypeVar

LocalMasterGraphClient = GraphClient
MasterGraphClient = GraphClient

DgraphClient: Any
DgraphClientStub: Any
Txn: Any
os: module

_T = TypeVar('_T')

class GraphClient(Any):
    txn_context: Callable[..., contextlib._GeneratorContextManager]
    def __init__(self) -> None: ...

def contextmanager(func: Callable[..., Iterator[_T]]) -> Callable[..., contextlib._GeneratorContextManager[_T]]: ...
def mg_alphas() -> Iterator[Tuple[str, int]]: ...
