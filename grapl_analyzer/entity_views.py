import json

from abc import ABC, abstractmethod
from typing import Optional, List, TypeVar, Dict, Any

from grapl_analyzer.entity_queries import FileQuery, ProcessQuery

from pydgraph import DgraphClient

# TODO: Replace DgraphClient with AnalyzerClient
#       We can then parameterize over that, and have an EngagementClient
#       which will allow for merging the libraries


P = TypeVar("P", bound="ProcessView")
N = TypeVar("N", bound="NodeView")
F = TypeVar("F", bound="FileView")


class NodeView(object):
    def as_process_view(self) -> Optional[P]:
        if isinstance(self, ProcessView):
            return self
        return None

    def as_file_view(self) -> Optional[F]:
        if isinstance(self, FileView):
            return self
        return None


class EdgeView(object):
    def __init__(
        self, from_neighbor_key: str, to_neighbor_key: str, edge_name: str
    ) -> None:
        self.from_neighbor_key = from_neighbor_key
        self.to_neighbor_key = to_neighbor_key
        self.edge_name = edge_name


class SubgraphView(object):
    def __init__(
        self, nodes: Dict[str, NodeView], edges: Dict[str, List[EdgeView]]
    ) -> None:
        self.nodes = nodes
        self.edges = edges

    def process_iter(self) -> Iterator[P]:
        for node in self.nodes.values():
            maybe_proc = node.as_process_view()
            if maybe_proc:
                yield proc

    def file_iter(self) -> Iterator[F]:
        for node in self.nodes.values():
            maybe_proc = node.as_file_view()
            if maybe_proc:
                yield proc


class ProcessView(NodeView):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: Optional[str],
        image_name: Optional[str],
        bin_file: Optional[F],
        parent: Optional[P],
        children: Optional[List[P]],
        deleted_files: Optional[List[F]],
        create_time: Optional[int],
        terminate_time: Optional[int],
    ) -> None:
        self.dgraph_client = dgraph_client  # type: DgraphClient
        self.node_key = node_key  # type: str
        self.uid = uid  # type: Optional[str]
        self.image_name = image_name  # type: Optional[str]
        self.bin_file = bin_file  # type: Optional[F]
        self.children = children  # type: Optional[List[P]]
        self.parent = parent  # type: Optional[P]
        self.deleted_files = deleted_files # type: Optional[List[F]]
        self.create_time = create_time  # type: Optional[int]
        self.terminate_time = terminate_time  # type: Optional[int]

    @staticmethod
    def from_dict(dgraph_client: DgraphClient, d: Dict[str, Any]) -> P:
        raw_bin_file = d.get("bin_file", None)

        bin_file = None

        if raw_bin_file:
            bin_file = FileView.from_dict(dgraph_client, raw_bin_file)

        raw_parent = d.get("~children", None)

        parent = None

        if raw_parent:
            parent = ProcessView.from_dict(dgraph_client, raw_parent)


        raw_deleted_files = d.get("deleted_files", None)

        deleted_files = None

        if raw_deleted_files:
            deleted_files = [
                FileView.from_dict(dgraph_client, f) for f in d['deleted_files']
            ]


        raw_children = d.get("children", None)

        children = None  # type: Optional[List[P]]
        if raw_children:
            children = [
                ProcessView.from_dict(dgraph_client, child) for child in d["children"]
            ]

        return ProcessView(
            dgraph_client=dgraph_client,
            node_key=d["node_key"],
            uid=d.get("uid", None),
            image_name=d.get("image_name", None),
            bin_file=bin_file,
            children=children,
            parent=parent,
            create_time=d.get("create_time", None),
            deleted_files=deleted_files,
            terminate_time=d.get("terminate_time", None),
        )

    def get_image_name(self) -> Optional[str]:
        if self.image_name:
            return self.image_name

        self_process = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_image_name()
                .query()
        )

        if not self_process:
            return None

        self.image_name = self_process[0].image_name
        return self.image_name

    def get_parent(self) -> Optional[P]:
        if self.parent:
            return self.parent

        query = (
            ProcessQuery()
            .with_child(ProcessQuery().with_node_key(eq=self.node_key))
            .with_uid()
            .to_query()
        )

        res = json.loads(self.dgraph_client.txn(read_only=True).query(query).json)

        parent = res["q0"]["~children"]
        self.parent = ProcessView.from_dict(self.dgraph_client, parent)
        return self.parent

    def get_uid(self):
        # type: () -> str
        if self.uid:
            return self.uid

        query = ProcessQuery().with_node_key(eq=self.node_key).with_uid().to_query()

        res = json.loads(self.dgraph_client.txn(read_only=True).query(query).json)

        uid = res["q0"]["uid"]
        assert uid
        self.uid = uid
        return uid

    def get_bin_file(self) -> Optional[F]:
        if self.bin_file:
            return self.bin_file

        query = (
            ProcessQuery()
            .with_node_key(eq=self.node_key)
            .with_bin_file(FileQuery())
            .to_query()
        )

        res = json.loads(
            self.dgraph_client
                .txn(read_only=True)
                .query(query)
                .json
        )

        bin_file = res["q0"]["bin_file"]
        self.bin_file = FileView.from_dict(self.dgraph_client, bin_file)
        return self.bin_file

    def get_deleted_files(self) -> Optional[List[F]]:
        if self.deleted_files:
            return self.deleted_files

        deleted_files = (
            ProcessQuery()
            .with_node_key(eq=self.node_key)
            .with_deleted_files(FileQuery().with_node_key())
            .query()
        )

        if not deleted_files:
            return None

        self.deleted_files = deleted_files[0].deleted_files
        return self.deleted_files


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
    def from_dict(dgraph_client: DgraphClient, d: Dict[str, Any]) -> F:
        return FileView(
            dgraph_client=dgraph_client,
            node_key=d["node_key"],
            uid=d.get("uid"),
            path=d.get("path"),
        )
