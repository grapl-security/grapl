import json

from abc import ABC, abstractmethod
from typing import Optional, List, TypeVar, Dict, Any

from grapl_analyzer.entity_queries import ProcessQuery

from pydgraph import DgraphClient

# TODO: Replace DgraphClient with AnalyzerClient
#       We can then parameterize over that, and have an EngagementClient
#       which will allow for merging the libraries


P = TypeVar('P', bound='ProcessView')
N = TypeVar('N', bound='NodeView')
F = TypeVar('F', bound='FileView')


class NodeView(object):

    @abstractmethod
    def as_process_view(self) -> Optional[P]:
        if isinstance(self, ProcessView):
            return self
        return None

    @abstractmethod
    def as_file_view(self) -> Optional[F]:
        if isinstance(self, FileView):
            return self
        return None


class EdgeView(object):
    def __init__(self,
                 from_neighbor_key: str,
                 to_neighbor_key: str,
                 edge_name: str,
                 ) -> None:
        self.from_neighbor_key = from_neighbor_key
        self.to_neighbor_key = to_neighbor_key
        self.edge_name = edge_name


class SubgraphView(object):
    def __init__(self,
                 nodes: Dict[str, NodeView],
                 edges: Dict[str, List[EdgeView]]
                 ) -> None:
        self.nodes = nodes
        self.edges = edges


class ProcessView(NodeView):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: Optional[str],
            image_name: Optional[str],
            bin_file: Optional[F],
            children: Optional[List[P]],
            create_time: Optional[int],
            terminate_time: Optional[int],
    ) -> None:
        self.dgraph_client = dgraph_client # type: DgraphClient
        self.node_key = node_key  # type: str
        self.uid = uid  # type: Optional[str]
        self.image_name = image_name  # type: Optional[str]
        self.bin_file = bin_file  # type: Optional[F]
        self.children = children  # type: Optional[List[P]]
        self.create_time = create_time  # type: Optional[int]
        self.terminate_time = terminate_time  # type: Optional[int]

    @staticmethod
    def from_dict(
            dgraph_client: DgraphClient,
            d: Dict[str, Any]
    ) -> P:
        raw_bin_file = d.get('bin_file', None)

        bin_file = None

        if raw_bin_file:
            bin_file = FileView.from_dict(dgraph_client, raw_bin_file)

        raw_children = d.get('children', None)

        children = None  # type: Optional[List[P]]
        if raw_children:
            children = \
                [ProcessView.from_dict(dgraph_client, child) for child in d['children']]

        return ProcessView(
            dgraph_client=dgraph_client,
            node_key=d['node_key'],
            uid=d.get('uid', None),
            image_name=d.get('image_name', None),
            bin_file=bin_file,
            children=children,
            create_time=d.get('create_time', None),
            terminate_time=d.get('terminate_time', None),
        )

    def get_uid(self):
        # type: () -> str
        if self.uid:
            return self.uid

        query = ProcessQuery()\
            .with_node_key(eq=self.node_key)\
            .with_uid()\
            .to_query()

        res = json.loads(
            self.dgraph_client
                .txn(read_only=True)
                .query(query)
                .json
        )

        uid = res['q0']['uid']
        assert uid
        self.uid = uid
        return uid


class FileView(NodeView):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: Optional[str],
            path: Optional[str],
    ) -> None:
        self.dgraph_client = dgraph_client  # type: DgraphClient
        self.node_key = node_key  # type: Optional[str]
        self.uid = uid  # type: Optional[str]
        self.path = path  # type: Optional[str]

    @staticmethod
    def from_dict(
            dgraph_client: DgraphClient,
            d: Dict[str, Any]
    ) -> F:
        return FileView(
            dgraph_client=dgraph_client,
            node_key=d['node_key'],
            uid=d.get('uid'),
            path=d.get('path'),
        )