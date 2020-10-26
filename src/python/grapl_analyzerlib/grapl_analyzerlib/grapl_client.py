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

    @classmethod
    def from_host_port(cls, host: str, port: int) -> "GraphClient":
        return cls(*(DgraphClientStub(f"{host}:{port}"),))

    @contextmanager
    def txn_context(self, read_only: bool = False) -> Iterator[Txn]:
        """
        Essentially, this just automates the try-finally in every
        txn() use case, turning it into a context manager.
        It'd be nice to - after a full migration to `txn_context` - perhaps restrict calls to `.txn()`
        """

        txn = self.txn(read_only=read_only)
        try:
            yield txn
        finally:
            txn.discard()


# These two classes were previously different, but now are unified by the MG_ALPHAS env variable.
# Consider them deprecated, and prefer GraphClient().
MasterGraphClient = GraphClient
LocalMasterGraphClient = GraphClient
