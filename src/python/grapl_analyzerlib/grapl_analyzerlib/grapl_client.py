import os

from typing import Iterator, List, Optional, Tuple
from grpc import CallCredentials

from pydgraph import DgraphClient, DgraphClientStub, Txn
from grapl_common.retry import retry
from contextlib import contextmanager

import pydgraph

from grapl_common.time_utils import SecsDuration

# https://dgraph.io/docs/clients/python/#setting-metadata-headers
DgraphMetadata = List[Tuple[str, str]]


def mg_alphas() -> Iterator[Tuple[str, int]]:
    # MG_ALPHAS being the list of "master graph alphas"
    # (master graph is an outdated term we don't really use in grapl anymore)
    # alpha being one of the Dgraph cluster's node types 
    # (https://dgraph.io/docs/get-started/#dgraph)
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

    @retry(ExceptionToCheck=pydgraph.RetriableError)
    def alter(
        self,
        operation: pydgraph.Operation,
        timeout: Optional[SecsDuration] = None,
        metadata: Optional[DgraphMetadata] = None,
        credentials: Optional[CallCredentials] = None,
    ) -> None:
        super(GraphClient, self).alter(
            operation=operation,
            timeout=timeout,
            metadata=metadata,
            credentials=credentials,
        )
