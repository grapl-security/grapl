import time
import unittest
import json
from copy import deepcopy
from typing import *

from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzerlib.nodes.comparators import IntCmp, _str_cmps, StrCmp, _int_cmps
from grapl_analyzerlib.nodes.dynamic_node import DynamicNodeQuery, DynamicNodeView
from grapl_analyzerlib.nodes.process_node import ProcessQuery, ProcessView, IProcessView
from grapl_analyzerlib.nodes.file_node import FileQuery, FileView
from grapl_analyzerlib.nodes.types import Property, PropertyT
from grapl_analyzerlib.nodes.viewable import Viewable, EdgeViewT, ForwardEdgeView


def _upsert(client: DgraphClient, node_dict: Dict[str, Property]) -> str:
    if node_dict.get('uid'):
        node_dict.pop('uid')
    node_dict['uid'] = '_:blank-0'
    node_key = node_dict['node_key']
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}")) {{
                    uid,  
                    expand(_all_)            
            }}
        }}
        """
    txn = client.txn(read_only=False)

    try:
        res = json.loads(txn.query(query).json)['q0']
        new_uid = None
        if res:
            node_dict['uid'] = res[0]['uid']
            new_uid = res[0]['uid']

        mutation = node_dict

        m_res = txn.mutate(set_obj=mutation, commit_now=True)
        uids = m_res.uids

        if new_uid is None:
            new_uid = uids['blank-0']
        return str(new_uid)

    finally:
        txn.discard()


def upsert(
        client: DgraphClient,
        type_name: str,
        view_type: Type[Viewable],
        node_key: str,
        node_props: Dict[str, Property]
) -> Viewable:
    node_props['node_key'] = node_key
    node_props['dgraph.type'] = type_name
    uid = _upsert(client, node_props)
    # print(f'uid: {uid}')
    node_props['uid'] = uid
    # print(node_props['node_key'])
    return view_type.from_dict(client, node_props)


def create_edge(client: DgraphClient, from_uid: str, edge_name: str, to_uid: str) -> None:
    if edge_name[0] == '~':
        mut = {
            'uid': to_uid,
            edge_name[1:]: {'uid': from_uid}
        }

    else:
        mut = {
            'uid': from_uid,
            edge_name: {'uid': to_uid}
        }

    txn = client.txn(read_only=False)
    try:
        txn.mutate(set_obj=mut, commit_now=True)
    finally:
        txn.discard()


class TestCase(unittest.TestCase):
    def test_get_basic_process_properties(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub('localhost:9080'))

        created_timestamp = int(time.time())
        terminate_time = created_timestamp + 5000
        
        process = {
            'process_id': 100,
            'process_name': 'word.exe',
            'image_name': 'word.exe',
            'created_timestamp': created_timestamp,
            'asset_id': 'asset_id-137',
            'terminate_time': terminate_time,
            'arguments': '--foo=bar baz',
        }  # type: Dict[str, Property]

        process_view = upsert(
            local_client,
            'Process',
            ProcessView,
            'ea75f056-61a1-479d-9ca2-f632d2c67205',
            process
        )

        # When we query for a process with pid 100 & process_name word.exe,
        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(eq="word.exe")
            .with_image_name(eq="word.exe")
            .with_created_timestamp(eq=created_timestamp)
            .with_asset_id(eq='asset_id-137')
            .with_arguments(eq='--foo=bar baz')
            .with_terminate_time(eq=terminate_time)
            .query_first(local_client)
        )

        # Then we will rec. a process view representing the process we inserted
        assert queried_process

    def test_process_name_contains(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub('localhost:9080'))

        created_timestamp = int(time.time())
        
        process = {
            'process_id': 100,
            'process_name': 'word.exe',
            'created_timestamp': created_timestamp
        }  # type: Dict[str, Property]

        process_view = upsert(
            local_client,
            'Process',
            ProcessView,
            'b5f4d54f-30c4-47c5-9e27-dcc1191aaf46',
            process
        )

        # When we query for a process with pid 100 & process_name word.exe,
        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .query_first(local_client)
        )

        # Then we will rec. a process view representing the process we inserted
        assert queried_process



    def test_parent_children_edge(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub('localhost:9080'))

        created_timestamp = int(time.time())
        
        parent_process = {
            'process_id': 100,
            'process_name': 'word.exe',
            'created_timestamp': created_timestamp
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            'Process',
            ProcessView,
            '0e84f2ce-f711-46ce-bc9e-1b13c9ba6d6c',
            parent_process
        )

        child_process = {
            'process_id': 110,
            'process_name': 'malware.exe',
            'created_timestamp': created_timestamp + 1000,
        }  # type: Dict[str, Property]

        child_process_view = upsert(
            local_client,
            'Process',
            ProcessView,
            '46d2862f-cb58-4062-b35e-bb310b8d5b0d',
            child_process
        )

        create_edge(
            local_client,
            parent_process_view.uid,
            'children',
            child_process_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_children(
                ProcessQuery()
                .with_process_id(eq=110)
                .with_process_name(eq='malware.exe')
                .with_created_timestamp(eq=created_timestamp + 1000)
            )
            .query_first(local_client)
        )


    def test_with_bin_file(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub('localhost:9080'))

        created_timestamp = int(time.time())
        
        parent_process = {
            'process_id': 100,
            'process_name': 'word.exe',
            'created_timestamp': created_timestamp
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            'Process',
            ProcessView,
            '635952af-87f3-4a2a-a65d-3f1859db9525',
            parent_process
        )

        bin_file = {
            'file_path': "/folder/file.txt",
            'created_timestamp': created_timestamp + 1000,
        }  # type: Dict[str, Property]

        bin_file_view = upsert(
            local_client,
            'File',
            FileView,
            '9f16e0c9-33c0-4d18-9878-ef686373570b',
            bin_file
        )

        create_edge(
            local_client,
            parent_process_view.uid,
            'bin_file',
            bin_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_bin_file(
                FileQuery()
                .with_file_path(eq="/folder/file.txt")
            )
            .query_first(local_client)
        )

        assert queried_process
        assert queried_process.process_id == 100

    def test_process_with_created_files(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub('localhost:9080'))

        created_timestamp = int(time.time())
        
        parent_process = {
            'process_id': 100,
            'process_name': 'word.exe',
            'created_timestamp': created_timestamp
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            'Process',
            ProcessView,
            '763ddbda-8812-4a07-acfe-83402b92379d',
            parent_process
        )

        created_file = {
            'file_path': "/folder/file.txt",
            'created_timestamp': created_timestamp + 1000,
        }  # type: Dict[str, Property]

        created_file_view = upsert(
            local_client,
            'File',
            FileView,
            '575f103e-1a11-4650-9f1b-5b72e44dfec3',
            created_file
        )

        create_edge(
            local_client,
            parent_process_view.uid,
            'created_files',
            created_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_created_files(
                FileQuery()
                .with_file_path(eq="/folder/file.txt")
            )
            .query_first(local_client)
        )

        assert queried_process
        assert queried_process.process_id == 100

    def test_with_deleted_files(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub('localhost:9080'))

        created_timestamp = int(time.time())
        
        parent_process = {
            'process_id': 100,
            'process_name': 'word.exe',
            'created_timestamp': created_timestamp
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            'Process',
            ProcessView,
            '47527d73-22c4-4e0f-bf7d-184bf1f206e2',
            parent_process
        )

        deleted_file = {
            'file_path': "/folder/file.txt",
            'created_timestamp': created_timestamp + 1000,
        }  # type: Dict[str, Property]

        deleted_file_view = upsert(
            local_client,
            'File',
            FileView,
            '8b8364ea-9b47-476b-8cf0-0f724adff10f',
            deleted_file
        )

        create_edge(
            local_client,
            parent_process_view.uid,
            'deleted_files',
            deleted_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_deleted_files(
                FileQuery()
                .with_file_path(eq="/folder/file.txt")
            )
            .query_first(local_client)
        )

        assert queried_process
        assert queried_process.process_id == 100

    def test_with_read_files(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub('localhost:9080'))

        created_timestamp = int(time.time())
        
        parent_process = {
            'process_id': 100,
            'process_name': 'word.exe',
            'created_timestamp': created_timestamp
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            'Process',
            ProcessView,
            '669a3693-d960-401c-8d29-5d669ffcd660',
            parent_process
        )

        read_file = {
            'file_path': "/folder/file.txt",
            'created_timestamp': created_timestamp + 1000,
        }  # type: Dict[str, Property]

        read_file_view = upsert(
            local_client,
            'File',
            FileView,
            'aa9248ec-36ee-4177-ba1a-999de735e682',
            read_file
        )

        create_edge(
            local_client,
            parent_process_view.uid,
            'read_files',
            read_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_read_files(
                FileQuery()
                .with_file_path(eq="/folder/file.txt")
            )
            .query_first(local_client)
        )

        assert queried_process
        assert queried_process.process_id == 100

    def test_with_wrote_files(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub('localhost:9080'))

        created_timestamp = int(time.time())
        
        parent_process = {
            'process_id': 100,
            'process_name': 'word.exe',
            'created_timestamp': created_timestamp
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            'Process',
            ProcessView,
            '8f0761fb-2ffe-4d4b-ab38-68e5489f56dc',
            parent_process
        )

        wrote_file = {
            'file_path': "/folder/file.txt",
            'created_timestamp': created_timestamp + 1000,
        }  # type: Dict[str, Property]

        wrote_file_view = upsert(
            local_client,
            'File',
            FileView,
            '2325c49a-95b4-423f-96d0-99539fe03833',
            wrote_file
        )

        create_edge(
            local_client,
            parent_process_view.uid,
            'wrote_files',
            wrote_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_read_files(
                FileQuery()
                .with_file_path(eq="/folder/file.txt")
            )
            .query_first(local_client)
        )

        assert queried_process
        assert queried_process.process_id == 100


if __name__ == '__main__':
    unittest.main()
