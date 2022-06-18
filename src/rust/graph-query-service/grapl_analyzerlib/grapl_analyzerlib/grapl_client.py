import os

from typing import Iterator, List, Optional, Tuple
from grpc import CallCredentials


from contextlib import contextmanager
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.retry import retry
from grapl_common.time_utils import SecsDuration

from query_gen import QueryGraphWithNodeRequest

LOGGER = get_module_grapl_logger()

# https://dgraph.io/docs/clients/python/#setting-metadata-headers
DgraphMetadata = List[Tuple[str, str]]


def query_hosts() -> Tuple[List[str], int]:
    hosts = os.environ["GRAPH_HOSTS"].split(",")
    port = int(os.environ["GRAPH_PORT"])
    return hosts, port




class GraphClient(object):
    def __init__(self) -> None:
        hosts, port = query_hosts()
        # Create the grpc client or the http client
        # todo: Tenant ID

    def query_with_uid(self, request: QueryGraphWithNodeRequest):
        ...


class GraphGrpcClient(GraphClient)
    ...