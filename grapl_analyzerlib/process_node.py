import json
from copy import deepcopy
from typing import Dict, List, Optional, Any, Tuple, Union

from pydgraph import DgraphClient

import grapl_analyzerlib.external_ip_node as external_ip_node 
import grapl_analyzerlib.file_node as file_node
from grapl_analyzerlib.node_types import FV, PQ, FQ, OCQ, EIPV, EIPQ, PV
import grapl_analyzerlib.outbound_connection_node as outbound_connection_node
from grapl_analyzerlib.querying import Has, Cmp, Queryable, Eq, _str_cmps, Viewable, PropertyFilter, Not, _int_cmps


class ProcessQuery(Queryable):
    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = []
        if self._asset_id:
            properties.append(("asset_id", self._asset_id))
        if self._process_name:
            properties.append(("process_name", self._process_name))
        if self._process_command_line:
            properties.append(("process_command_line", self._process_command_line))
        if self._process_guid:
            properties.append(("process_guid", self._process_guid))
        if self._process_id:
            properties.append(("process_id", self._process_id))
        if self._created_timestamp:
            properties.append(("created_timestamp", self._created_timestamp))
        if self._terminated_timestamp:
            properties.append(("terminated_timestamp", self._terminated_timestamp))
        if self._last_seen_timestamp:
            properties.append(("last_seen_timestamp", self._last_seen_timestamp))

        properties.append(('node_key', self.get_node_key_filter()))
        properties.append(('uid', self.get_uid_filter()))

        return properties

    def get_node_type_name(self) -> Optional[str]:
        return None

    def get_unique_predicate(self) -> Optional[str]:
        return 'process_id'

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        return [[self._uid]]

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        edges = [
            ("bin_file", self._bin_file) if self._bin_file else None,
            ("children", self._children) if self._children else None,
            ("deleted_files", self._deleted_files) if self._deleted_files else None,
            ("created_files", self._created_files) if self._created_files else None,
            ("wrote_to_files", self._wrote_to_files) if self._wrote_to_files else None,
            ("read_files", self._read_files) if self._read_files else None,
            ("created_connection", self._created_connection) if self._created_connection else None,
        ]

        return [e for e in edges if e]

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        edges = [
            ("~children", self._parent) if self._parent else None,
        ]

        return [e for e in edges if e]


    def __init__(self) -> None:
        super(ProcessQuery, self).__init__()
        # Properties
        self._node_key = Has(
            "node_key"
        )  # type: Cmp
        self._uid = Has(
            "uid"
        )  # type: Cmp

        self._asset_id = []  # type: List[List[Cmp]]
        self._process_name = []  # type: List[List[Cmp]]
        self._process_command_line = []  # type: List[List[Cmp]]
        self._process_guid = []  # type: List[List[Cmp]]
        self._process_id = []  # type: List[List[Cmp]]
        self._created_timestamp = []  # type: List[List[Cmp]]
        self._terminated_timestamp = []  # type: List[List[Cmp]]
        self._last_seen_timestamp = []  # type: List[List[Cmp]]

        # Edges
        self._bin_file = None  # type: Optional[FQ]
        self._children = None  # type: Optional[PQ]
        self._deleted_files = None  # type: Optional[FQ]
        self._created_files = None  # type: Optional[FQ]
        self._wrote_to_files = None  # type: Optional[FQ]
        self._read_files = None  # type: Optional[FQ]
        self._created_connection = None  # type: Optional[OCQ]

        self._parent = None  # type: Optional[PQ]

        # Meta
        self._first = None  # type: Optional[int]

    def with_node_key(self, node_key: Optional[Union[Not, str]] = None):
        if node_key:
            self._node_key = Eq("node_key", node_key)
        else:
            self._node_key = Has("node_key")
        return self

    def with_uid(self, uid: Optional[Union[Not, str]] = None):
        if uid:
            self._uid = Eq("uid", uid)
        else:
            self._uid = Has("uid")
        return self

    def only_first(self, first: int) -> PQ:
        self._first = first
        return self

    def with_asset_id(
            self,
            eq: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            contains: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            ends_with: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
    ) -> PQ:
        self._asset_id.extend(
            _str_cmps("asset_id", eq, contains, ends_with)
        )
        return self

    def with_process_name(
            self,
            eq: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            contains: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            ends_with: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
    ) -> PQ:
        self._process_name.extend(
            _str_cmps("process_name", eq, contains, ends_with)
        )
        return self

    def with_process_command_line(
            self,
            eq: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            contains: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            ends_with: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
    ) -> PQ:
        self._process_command_line.extend(
            _str_cmps("process_command_line", eq, contains, ends_with)
        )
        return self

    def with_process_guid(
            self,
            eq: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            contains: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
            ends_with: Optional[
                Union[str, List[str], Not, List[Not]]
            ] = None,
    ) -> PQ:
        self._process_guid.extend(
            _str_cmps("process_guid", eq, contains, ends_with)
        )
        return self

    def with_process_id(
            self,
            eq: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            gt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            lt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
    ) -> PQ:
        self._process_id.extend(_int_cmps("process_id", eq, gt, lt))
        return self

    def with_created_timestamp(
            self,
            eq: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            gt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            lt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
    ) -> PQ:
        self._created_timestamp.extend(
            _int_cmps("created_timestamp", eq, gt, lt)
        )
        return self

    def with_terminated_timestamp(
            self,
            eq: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            gt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            lt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
    ) -> PQ:
        self._terminated_timestamp.extend(
            _int_cmps("terminated_timestamp", eq, gt, lt)
        )
        return self

    def with_last_seen_timestamp(
            self,
            eq: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            gt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
            lt: Optional[
                Union[str, List[int], Not, List[Not]]
            ] = None,
    ) -> PQ:
        self._last_seen_timestamp.extend(
            _int_cmps("last_seen_timestamp", eq, gt, lt)
        )
        return self

    def with_parent(self, process: PQ) -> PQ:
        process: PQ = deepcopy(process)
        process._children = self
        self._parent = process
        return self

    def with_bin_file(self, file: FQ) -> PQ:
        file = deepcopy(file)
        file._spawned_from = self
        self._bin_file = file
        return self

    def with_deleted_files(self, file: FQ) -> PQ:
        file = deepcopy(file)
        file._deleter = self
        self._deleted_files = file
        return self

    def with_created_files(self, file: FQ) -> PQ:
        file = deepcopy(file)
        file._creator = self
        self._created_files = file
        return self

    def with_written_files(self, file: FQ) -> PQ:
        file = deepcopy(file)
        file._writers = self
        self._wrote_to_files = file
        return self

    def with_read_files(self, file: FQ) -> PQ:
        file = deepcopy(file)
        file._readers = self
        self._read_files = file
        return self

    def with_children(self, children: PQ) -> PQ:
        children = deepcopy(children)
        children._parent = self
        self._children = children
        return self

    def with_created_connection(
            self,
            outbound_conn: Union[OCQ, EIPQ]
    ) -> PQ:
        outbound_conn = deepcopy(outbound_conn)

        if isinstance(outbound_conn, external_ip_node.ExternalIpQuery):
            outbound_conn = (
                outbound_connection_node.OutboundConnectionQuery()
                .with_external_connection(outbound_conn)
            )
        outbound_conn._connecting_process = self

        self._created_connection = outbound_conn
        return self

    def query_first(self, dgraph_client, contains_node_key=None) -> Optional[PV]:
        return super(ProcessQuery, self)._query_first(
            dgraph_client, ProcessView, contains_node_key
        )


class ProcessView(Viewable):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: Optional[str] = None,
            asset_id: Optional[str] = None,
            process_name: Optional[str] = None,
            process_command_line: Optional[str] = None,
            process_guid: Optional[str] = None,
            process_id: Optional[str] = None,
            created_timestamp: Optional[str] = None,
            terminated_timestamp: Optional[str] = None,
            last_seen_timestamp: Optional[str] = None,
            bin_file: Optional[FV] = None,
            parent: Optional[PV] = None,
            children: Optional[List[PV]] = None,
            deleted_files: Optional[List[FV]] = None,
            created_files: Optional[List[FV]] = None,
            read_files: Optional[List[FV]] = None,
            created_connections: Optional[List[EIPV]] = None,
    ) -> None:
        super(ProcessView, self).__init__(self)

        self.dgraph_client = dgraph_client  # type: DgraphClient
        self.node_key = node_key  # type: str
        self.uid = uid  # type: Optional[str]
        self.asset_id = asset_id
        self.process_command_line = process_command_line
        self.process_guid = process_guid
        self.process_id = process_id
        self.created_timestamp = created_timestamp
        self.terminated_timestamp = terminated_timestamp
        self.last_seen_timestamp = last_seen_timestamp
        self.process_name = process_name  # type: Optional[str]

        # TODO: Support created, deleted, written, read
        self.bin_file = bin_file  # type: Optional[FV]
        self.children = children  # type: Optional[List[PV]]
        self.parent = parent  # type: Optional[PV]
        self.deleted_files = deleted_files  # type: Optional[List[FV]]
        self.created_files = created_files  # type: Optional[List[FV]]
        self.read_files = read_files  # type: Optional[List[FV]]
        self.created_connections = created_connections  # type: Optional[List[EIPV]]

    @staticmethod
    def from_dict(dgraph_client: DgraphClient, d: Dict[str, Any]) -> PV:
        raw_created_connections = d.get("created_connection", None)

        created_connections = None

        if raw_created_connections:
            created_connections = [
                OutboundConnectionView.from_dict(dgraph_client, c)
                for c in raw_created_connections
            ]


        raw_bin_file = d.get("bin_file", None)

        bin_file = None

        if raw_bin_file:
            bin_file = file_node.FileView.from_dict(dgraph_client, raw_bin_file[0])

        raw_parent = d.get("~children", None)

        parent = None

        if raw_parent:
            parent = ProcessView.from_dict(dgraph_client, raw_parent[0])

        raw_deleted_files = d.get("deleted_files", None)

        deleted_files = None

        if raw_deleted_files:
            deleted_files = [
                file_node.FileView.from_dict(dgraph_client, f) for f in d["deleted_files"]
            ]

        raw_read_files = d.get("read_files", None)

        read_files = None

        if raw_read_files:
            read_files = [
                file_node.FileView.from_dict(dgraph_client, f) for f in d["read_files"]
            ]

        raw_created_files = d.get("created_files", None)

        created_files = None

        if raw_created_files:
            created_files = [
                file_node.FileView.from_dict(dgraph_client, f) for f in d["created_files"]
            ]

        raw_children = d.get("children", None)

        children = None  # type: Optional[List[PV]]
        if raw_children:
            children = [
                ProcessView.from_dict(dgraph_client, child) for child in d["children"]
            ]

        return ProcessView(
            dgraph_client=dgraph_client,
            node_key=d["node_key"],
            uid=d["uid"],
            asset_id=d.get("asset_id"),
            process_name=d.get("process_name"),
            process_command_line=d.get("process_command_line"),
            process_guid=d.get("process_guid"),
            process_id=d.get("process_id"),
            created_timestamp=d.get("created_timestamp"),
            terminated_timestamp=d.get("terminated_timestamp"),
            last_seen_timestamp=d.get("last_seen_timestamp"),
            bin_file=bin_file,
            deleted_files=deleted_files,
            read_files=read_files,
            created_files=created_files,
            children=children,
            parent=parent,
            created_connections=created_connections,
        )

    def get_asset_id(self) -> Optional[str]:
        if self.asset_id:
            return self.asset_id

        self_process = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_asset_id()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_process:
            return None

        self.asset_id = self_process.asset_id
        return self.asset_id

    def get_process_name(self) -> Optional[str]:
        if self.process_name:
            return self.process_name

        self_process = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_process_name()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_process:
            return None

        self.process_name = self_process.process_name
        return self.process_name

    def get_process_command_line(self) -> Optional[str]:
        if self.process_command_line:
            return self.process_command_line

        self_process = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_process_command_line()
                .query_first(dgraph_client=self.dgraph_client)
        )

        if not self_process:
            return None

        self.process_command_line = self_process.process_command_line
        return self.process_command_line

    def get_parent(self) -> Optional[PV]:
        if self.parent:
            return self.parent

        parent = (
            ProcessQuery()
                .with_children(ProcessQuery().with_node_key(self.node_key))
                .query_first(self.dgraph_client)
        )

        if not parent:
            return None

        self.parent = parent
        return self.parent

    def get_children(self) -> Optional[List[PV]]:
        if self.children:
            return self.children

        self_node = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_children(ProcessQuery().with_node_key())
                .query_first(self.dgraph_client)
        )  # type: PV

        self.children = self_node.children or None

        return self.children

    def get_uid(self):
        # type: () -> str
        if self.uid:
            return self.uid

        process = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_uid()
                .query_first(self.dgraph_client)
        )

        assert process
        self.uid = process.uid
        return process.uid

    def get_bin_file(self) -> Optional[FV]:
        if self.bin_file:
            return self.bin_file

        query = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_bin_file(file_node.FileQuery())
                .to_query()
        )

        res = json.loads(self.dgraph_client.txn(read_only=True).query(query).json)

        bin_file = res["q0"]["bin_file"]
        self.bin_file = file_node.FileView.from_dict(self.dgraph_client, bin_file[0])
        return self.bin_file

    def get_deleted_files(self) -> Optional[List[FV]]:
        if self.deleted_files:
            return self.deleted_files

        deleted_files = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_deleted_files(file_node.FileQuery().with_node_key())
                .query()
        )

        if not deleted_files:
            return None

        self.deleted_files = deleted_files[0].deleted_files
        return self.deleted_files

    def get_read_files(self) -> Optional[List[FV]]:
        if self.read_files:
            return self.read_files

        read_files = (
            ProcessQuery()
                .with_node_key(self.node_key)
                .with_read_files(file_node.FileQuery().with_node_key())
                .query()
        )

        if not read_files:
            return None

        self.read_files = read_files[0].read_files
        return self.read_files

    def get_neighbors(self) -> List[Any]:
        neighbors = (self.parent, self.bin_file, self.children, self.deleted_files)

        return [n for n in neighbors if n]

    def get_edges(self) -> List[Tuple[str, Any]]:
        neighbors = (
            ("parent", self.parent) if self.parent else None,
            ("bin_file", self.bin_file) if self.bin_file else None,
            ("children", self.children) if self.children else None,
            ("deleted_files", self.deleted_files) if self.deleted_files else None,
        )

        return [n for n in neighbors if n]

    def to_dict(self, root=False) -> Dict[str, Any]:
        node_dict = dict()
        edges = []
        node_dict['node_type'] = 'Process'
        if self.node_key:
            node_dict['node_key'] = self.node_key
        if self.uid:
            node_dict['uid'] = self.uid
        if self.process_command_line:
            node_dict['process_command_line'] = self.process_command_line
        if self.process_guid:
            node_dict['process_guid'] = self.process_guid
        if self.process_id:
            node_dict['process_id'] = self.process_id
        if self.created_timestamp:
            node_dict['created_timestamp'] = self.created_timestamp
        if self.terminated_timestamp:
            node_dict['terminated_timestamp'] = self.terminated_timestamp
        if self.last_seen_timestamp:
            node_dict['last_seen_timestamp'] = self.last_seen_timestamp
        if self.process_name:
            node_dict['process_name'] = self.process_name

        if self.asset_id:
            node_dict['process_name'] = self.asset_id

        if self.bin_file:
            node_dict['bin_file'] = self.bin_file.node_key
            edges.append(
                {
                    'from': self.node_key,
                    'edge_name': 'bin_file',
                    'to': self.bin_file.node_key
                }
            )

        if self.children:
            for child in self.children:
                edges.append(
                    {
                        'from': self.node_key,
                        'edge_name': 'children',
                        'to': child.node_key
                    }
                )
        if self.parent:
            edges.append(
                {
                    'from': self.parent.node_key,
                    'edge_name': 'children',
                    'to': self.node_key
                }
            )

        if self.deleted_files:
            for deleted_file in self.deleted_files:
                edges.append(
                    {
                        'from': self.node_key,
                        'edge_name': 'deleted_files',
                        'to': deleted_file.node_key
                    }
                )
        if root:
            node_dict['root'] = True

        return {'node': node_dict, 'edges': edges}


if __name__ == '__main__':
    ProcessQuery().query_first(
        None, "119895c8-f222-4e0b-906a-b33149ce6de1"
    )