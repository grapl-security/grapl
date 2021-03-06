import os

from typing import Iterator, Tuple

from pydgraph import DgraphClient, DgraphClientStub, Txn
from contextlib import contextmanager


def mg_alphas() -> Iterator[Tuple[str, int]]:
    mg_alphas = os.environ["MG_ALPHAS"].split(",")
    for mg_alpha in mg_alphas:
        host, port = mg_alpha.split(":")
        yield host, int(port)


class GraphClient(DgraphClient):
    def __init__(self) -> None:
        super(GraphClient, self).__init__(
            *(DgraphClientStub(f"{host}:{port}") for host, port in mg_alphas())
        )

    @contextmanager
    def txn_context(
        self,
        read_only: bool = False,
        best_effort: bool = False,
    ) -> Iterator[Txn]:
        """
        Essentially, this just automates the try-finally in every
        txn() use case, turning it into a context manager.
        It'd be nice to - after a full migration to `txn_context` - perhaps restrict calls to `.txn()`
        """

        txn = self.txn(read_only=read_only, best_effort=best_effort)
        try:
            yield txn
        finally:
            txn.discard()
