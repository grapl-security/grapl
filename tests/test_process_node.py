import json
import time
import unittest
from typing import *

import hypothesis
import hypothesis.strategies as st
from hypothesis import given
from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzerlib.nodes.comparators import escape_dgraph_str, Not
from grapl_analyzerlib.nodes.file_node import FileQuery, FileView
from grapl_analyzerlib.nodes.process_node import ProcessQuery, ProcessView
from grapl_analyzerlib.nodes.types import Property
from grapl_analyzerlib.nodes.viewable import Viewable
from grapl_analyzerlib.nodes.asset_node import AssetView, AssetQuery
from test_utils.dgraph_utils import upsert, create_edge, node_key_for_test


def get_or_create_asset_node(
    local_client: DgraphClient,
    node_key: str,
    # AssetView properties
    hostname: str,
) -> AssetView:
    node_props = {
        "hostname": hostname,
    }  # type: Dict[str, Property]

    return cast(
        AssetView, upsert(local_client, "Asset", AssetView, node_key, node_props)
    )


def get_or_create_process_node(
    local_client: DgraphClient,
    node_key: str,
    # properties
    process_id: str,
    arguments: str,
    created_timestamp: str,
    # asset_id: str,
    terminate_time: str,
    image_name: str,
    process_name: str,
) -> ProcessView:
    node_props = {
        "process_id": process_id,
        "arguments": arguments,
        "created_timestamp": created_timestamp,
        # "asset_id": asset_id,
        "terminate_time": terminate_time,
        "image_name": image_name,
        "process_name": process_name,
    }  # type: Dict[str, Property]

    return cast(
        ProcessView, upsert(local_client, "Process", ProcessView, node_key, node_props)
    )


class TestProcessQuery(unittest.TestCase):

    # @classmethod
    # def setUpClass(cls):
    #     local_client = DgraphClient(DgraphClientStub('localhost:9080'))
    #
    #     # drop_all(local_client)
    #     # time.sleep(3)
    #     provision()
    #     provision()

    # @classmethod
    # def tearDownClass(cls):
    #     local_client = DgraphClient(DgraphClientStub('localhost:9080'))
    #
    #     drop_all(local_client)
    #     provision()

    @hypothesis.settings(deadline=None)
    @given(
        process_node_key=st.uuids(),
        process_id=st.integers(min_value=1, max_value=2 ** 32),
        created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
        terminate_time=st.integers(min_value=0, max_value=2 ** 48),
        image_name=st.text(),
        process_name=st.text(),
        arguments=st.text(),
    )
    def test_single_process_contains_key(
        self,
        process_node_key,
        process_id,
        created_timestamp,
        terminate_time,
        image_name,
        process_name,
        arguments,
    ):
        process_node_key = node_key_for_test(self, process_node_key)
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))

        created_process_node = get_or_create_process_node(
            local_client,
            process_node_key,
            process_id,
            arguments,
            created_timestamp,
            terminate_time,
            image_name,
            process_name,
        )

        # Setup complete, do some queries

        queried_proc = ProcessQuery().query_first(
            local_client, contains_node_key=process_node_key
        )

        assert process_id == queried_proc.get_process_id()
        assert process_node_key == queried_proc.node_key
        assert "Process" == queried_proc.get_node_type()
        assert process_id == queried_proc.get_process_id()
        assert arguments == queried_proc.get_arguments()
        assert created_timestamp == queried_proc.get_created_timestamp()
        assert terminate_time == queried_proc.get_terminate_time()
        assert image_name == queried_proc.get_image_name()
        assert process_name == queried_proc.get_process_name()

        assert not queried_proc.get_asset()

    @hypothesis.settings(deadline=None)
    @given(
        asset_node_key=st.uuids(),
        hostname=st.text(),
        process_node_key=st.uuids(),
        process_id=st.integers(min_value=1, max_value=2 ** 32),
        created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
        terminate_time=st.integers(min_value=0, max_value=2 ** 48),
        image_name=st.text(),
        process_name=st.text(),
        arguments=st.text(),
    )
    def test_single_process_connected_to_asset_node(
        self,
        asset_node_key,
        hostname,
        process_node_key,
        process_id,
        created_timestamp,
        terminate_time,
        image_name,
        process_name,
        arguments,
    ):
        asset_node_key = node_key_for_test(self, asset_node_key)
        process_node_key = node_key_for_test(self, process_node_key)
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))

        created_asset_node = get_or_create_asset_node(
            local_client, asset_node_key, hostname=hostname
        )

        created_process_node = get_or_create_process_node(
            local_client,
            process_node_key,
            process_id,
            arguments,
            created_timestamp,
            terminate_time,
            image_name,
            process_name,
        )

        create_edge(
            local_client,
            created_asset_node.uid,
            "asset_processes",
            created_process_node.uid,
        )

        # Setup complete, do some queries

        queried_proc = (
            ProcessQuery()
            .with_asset(AssetQuery().with_hostname(hostname))
            .query_first(local_client, contains_node_key=process_node_key)
        )

        assert process_id == queried_proc.get_process_id()
        assert process_node_key == queried_proc.node_key
        assert "Process" == queried_proc.get_node_type()
        assert process_id == queried_proc.get_process_id()
        assert arguments == queried_proc.get_arguments()
        assert created_timestamp == queried_proc.get_created_timestamp()
        assert terminate_time == queried_proc.get_terminate_time()
        assert image_name == queried_proc.get_image_name()
        assert process_name == queried_proc.get_process_name()

        queried_asset = queried_proc.get_asset()
        assert queried_asset
        assert queried_asset.node_key == asset_node_key

    # Given that the code that generates timestamps only uses unsized types we can make some
    # assumptions about the data
    @hypothesis.settings(deadline=None)
    @given(
        node_key=st.uuids(),
        process_id=st.integers(min_value=1, max_value=2 ** 32),
        created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
        asset_id=st.text(),
        terminate_time=st.integers(min_value=0, max_value=2 ** 48),
        image_name=st.text(),
        process_name=st.text(),
        arguments=st.text(),
    )
    def test_process_query_view_parity(
        self,
        node_key,
        process_id,
        created_timestamp,
        asset_id,
        terminate_time,
        image_name,
        process_name,
        arguments,
    ):
        node_key = "test_process_query_view_parity" + str(node_key)
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))

        get_or_create_process_node(
            local_client,
            node_key,
            process_id,
            arguments,
            created_timestamp,
            asset_id,
            terminate_time,
            image_name,
            process_name,
        )

        queried_proc = (
            ProcessQuery().with_node_key(eq=node_key).query_first(local_client)
        )

        # assert process_view.process_id == queried_proc.get_process_id()
        assert node_key == queried_proc.node_key
        assert "Process" == queried_proc.get_node_type()
        assert process_id == queried_proc.get_process_id()
        assert arguments == queried_proc.get_arguments()
        assert created_timestamp == queried_proc.get_created_timestamp()
        assert asset_id == queried_proc.get_asset_id()
        assert terminate_time == queried_proc.get_terminate_time()
        assert image_name == queried_proc.get_image_name()
        assert process_name == queried_proc.get_process_name()

    @hypothesis.settings(deadline=None)
    @given(
        node_key=st.uuids(),
        process_id=st.integers(min_value=1, max_value=2 ** 32),
        created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
        asset_id=st.text(),
        terminate_time=st.integers(min_value=0, max_value=2 ** 48),
        image_name=st.text(),
        process_name=st.text(),
        arguments=st.text(),
    )
    def test_process_query_view_parity_eq(
        self,
        node_key,
        process_id,
        created_timestamp,
        asset_id,
        terminate_time,
        image_name,
        process_name,
        arguments,
    ):
        node_key = "test_process_query_view_parity_eq" + str(node_key)
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))
        get_or_create_process_node(
            local_client,
            node_key,
            process_id,
            arguments,
            created_timestamp,
            asset_id,
            terminate_time,
            image_name,
            process_name,
        )

        queried_proc = (
            ProcessQuery()
            .with_node_key(eq=node_key)
            .with_process_id(eq=process_id)
            .with_arguments(eq=arguments)
            .with_created_timestamp(eq=created_timestamp)
            .with_asset_id(eq=asset_id)
            .with_terminate_time(eq=terminate_time)
            .with_image_name(eq=image_name)
            .with_process_name(eq=process_name)
            .query_first(local_client)
        )

        # assert process_view.process_id == queried_proc.get_process_id()
        assert node_key == queried_proc.node_key
        assert "Process" == queried_proc.get_node_type()
        assert process_id == queried_proc.get_process_id()

        assert arguments == queried_proc.get_arguments()
        assert created_timestamp == queried_proc.get_created_timestamp()
        assert asset_id == queried_proc.get_asset_id()
        assert terminate_time == queried_proc.get_terminate_time()
        assert image_name == queried_proc.get_image_name()
        assert process_name == queried_proc.get_process_name()

    @hypothesis.settings(deadline=None)
    @given(
        node_key=st.uuids(),
        process_id=st.integers(min_value=1, max_value=2 ** 32),
        created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
        asset_id=st.text(),
        terminate_time=st.integers(min_value=0, max_value=2 ** 48),
        image_name=st.text(),
        process_name=st.text(),
        arguments=st.text(),
    )
    def test_process_query_view_miss(
        self,
        node_key,
        process_id,
        created_timestamp,
        asset_id,
        terminate_time,
        image_name,
        process_name,
        arguments,
    ):
        node_key = "test_process_query_view_miss" + str(node_key)
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))
        process = {
            "process_id": process_id,
            "arguments": arguments,
            "created_timestamp": created_timestamp,
            "asset_id": asset_id,
            "terminate_time": terminate_time,
            "image_name": image_name,
            "process_name": process_name,
        }  # type: Dict[str, Property]

        process_view = cast(
            ProcessView, upsert(local_client, "Process", ProcessView, node_key, process)
        )  # type: ProcessView

        queried_proc = (
            ProcessQuery()
            .with_node_key(eq=node_key)
            .with_process_id(eq=Not(process_id))
            .with_arguments(eq=Not(arguments))
            .with_created_timestamp(eq=Not(created_timestamp))
            .with_asset_id(eq=Not(asset_id))
            .with_terminate_time(eq=Not(terminate_time))
            .with_image_name(eq=Not(image_name))
            .with_process_name(eq=Not(process_name))
            .query_first(local_client)
        )

        assert not queried_proc

    # Given that the code that generates timestamps only uses unsized types we can make some
    # assumptions about the data

    @hypothesis.settings(deadline=None)
    @given(
        node_key=st.uuids(),
        process_id=st.integers(min_value=1, max_value=2 ** 32),
        created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
        asset_id=st.text(),
        terminate_time=st.integers(min_value=0, max_value=2 ** 48),
        image_name=st.text(),
        process_name=st.text(),
        arguments=st.text(),
    )
    def test_process_query_view_parity_contains(
        self,
        node_key,
        process_id,
        created_timestamp,
        asset_id,
        terminate_time,
        image_name,
        process_name,
        arguments,
    ):
        node_key = "test_process_query_view_parity_contains" + str(node_key)
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))
        get_or_create_process_node(
            local_client,
            node_key,
            process_id,
            arguments,
            created_timestamp,
            asset_id,
            terminate_time,
            image_name,
            process_name,
        )

        query = ProcessQuery().with_node_key(eq=node_key)

        # Don't fuck with newlines due to a dgraph bug
        # https://github.com/dgraph-io/dgraph/issues/4694
        if len(arguments) > 3 and "\n" not in arguments:
            query.with_arguments(contains=arguments[: len(arguments) - 1])
        if len(asset_id) > 3 and "\n" not in asset_id:
            query.with_asset_id(contains=asset_id[: len(asset_id) - 1])
        if len(image_name) > 3 and "\n" not in image_name:
            query.with_image_name(contains=image_name[: len(image_name) - 1])
        if len(process_name) > 3 and "\n" not in process_name:
            query.with_process_name(contains=process_name[: len(process_name) - 1])

        queried_proc = query.query_first(local_client)

        assert queried_proc
        assert "Process" == queried_proc.get_node_type()
        assert process_id == queried_proc.get_process_id()
        assert node_key == queried_proc.node_key
        assert arguments == queried_proc.get_arguments()
        assert created_timestamp == queried_proc.get_created_timestamp()
        assert asset_id == queried_proc.get_asset_id()
        assert terminate_time == queried_proc.get_terminate_time()
        assert image_name == queried_proc.get_image_name()
        assert process_name == queried_proc.get_process_name()

    def test_parent_children_edge(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            "Process",
            ProcessView,
            "0e84f2ce-f711-46ce-bc9e-1b13c9ba6d6c",
            parent_process,
        )

        child_process = {
            "process_id": 110,
            "process_name": "malware.exe",
            "created_timestamp": created_timestamp + 1000,
        }  # type: Dict[str, Property]

        child_process_view = upsert(
            local_client,
            "Process",
            ProcessView,
            "46d2862f-cb58-4062-b35e-bb310b8d5b0d",
            child_process,
        )

        create_edge(
            local_client, parent_process_view.uid, "children", child_process_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_node_key(eq="0e84f2ce-f711-46ce-bc9e-1b13c9ba6d6c")
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_children(
                ProcessQuery()
                .with_node_key(eq="46d2862f-cb58-4062-b35e-bb310b8d5b0d")
                .with_process_id(eq=110)
                .with_process_name(eq="malware.exe")
                .with_created_timestamp(eq=created_timestamp + 1000)
            )
            .query_first(local_client)
        )
        assert queried_process

        assert queried_process.node_key == "0e84f2ce-f711-46ce-bc9e-1b13c9ba6d6c"
        assert queried_process.process_id == 100
        assert queried_process.process_name == "word.exe"
        assert queried_process.created_timestamp == created_timestamp

        assert len(queried_process.children) == 1
        child = queried_process.children[0]
        assert child.node_key == "46d2862f-cb58-4062-b35e-bb310b8d5b0d"
        assert child.process_id == 110
        assert child.process_name == "malware.exe"
        assert child.created_timestamp == created_timestamp + 1000

    def test_with_bin_file(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            "Process",
            ProcessView,
            "635952af-87f3-4a2a-a65d-3f1859db9525",
            parent_process,
        )

        bin_file = {
            "file_path": "/folder/file.txt",
            "created_timestamp": created_timestamp + 1000,
        }  # type: Dict[str, Property]

        bin_file_view = upsert(
            local_client,
            "File",
            FileView,
            "9f16e0c9-33c0-4d18-9878-ef686373570b",
            bin_file,
        )

        create_edge(
            local_client, parent_process_view.uid, "bin_file", bin_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_node_key(eq="635952af-87f3-4a2a-a65d-3f1859db9525")
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_bin_file(
                FileQuery()
                .with_node_key(eq="9f16e0c9-33c0-4d18-9878-ef686373570b")
                .with_file_path(eq="/folder/file.txt")
            )
            .query_first(local_client)
        )

        assert queried_process
        assert "635952af-87f3-4a2a-a65d-3f1859db9525"
        assert queried_process.process_id == 100
        assert queried_process.process_name == "word.exe"
        assert queried_process.created_timestamp == created_timestamp

        bin_file = queried_process.bin_file
        assert bin_file.node_key == "9f16e0c9-33c0-4d18-9878-ef686373570b"

        assert bin_file.file_path == "/folder/file.txt"

    def test_process_with_created_files(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            "Process",
            ProcessView,
            "763ddbda-8812-4a07-acfe-83402b92379d",
            parent_process,
        )

        created_file = {
            "file_path": "/folder/file.txt",
            "created_timestamp": created_timestamp + 1000,
        }  # type: Dict[str, Property]

        created_file_view = upsert(
            local_client,
            "File",
            FileView,
            "575f103e-1a11-4650-9f1b-5b72e44dfec3",
            created_file,
        )

        create_edge(
            local_client,
            parent_process_view.uid,
            "created_files",
            created_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_node_key(eq="763ddbda-8812-4a07-acfe-83402b92379d")
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_created_files(
                FileQuery()
                .with_node_key(eq="575f103e-1a11-4650-9f1b-5b72e44dfec3")
                .with_file_path(eq="/folder/file.txt")
            )
            .query_first(local_client)
        )

        assert queried_process
        assert queried_process.process_id == 100

        assert len(queried_process.created_files) == 1
        created_file = queried_process.created_files[0]
        assert created_file.file_path == "/folder/file.txt"

    def test_with_deleted_files(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            "Process",
            ProcessView,
            "test_with_deleted_files-47527d73-22c4-4e0f-bf7d-184bf1f206e2",
            parent_process,
        )

        deleted_file = {
            "file_path": "/folder/file.txt",
            "created_timestamp": created_timestamp + 1000,
        }  # type: Dict[str, Property]

        deleted_file_view = upsert(
            local_client,
            "File",
            FileView,
            "test_with_deleted_files8b8364ea-9b47-476b-8cf0-0f724adff10f",
            deleted_file,
        )

        create_edge(
            local_client,
            parent_process_view.uid,
            "deleted_files",
            deleted_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_deleted_files(FileQuery().with_file_path(eq="/folder/file.txt"))
            .query_first(local_client)
        )

        assert queried_process
        assert queried_process.process_id == 100

    def test_with_read_files(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            "Process",
            ProcessView,
            "test_with_read_files-669a3693-d960-401c-8d29-5d669ffcd660",
            parent_process,
        )

        read_file = {
            "file_path": "/folder/file.txt",
            "created_timestamp": created_timestamp + 1000,
        }  # type: Dict[str, Property]

        read_file_view = upsert(
            local_client,
            "File",
            FileView,
            "test_with_read_files-aa9248ec-36ee-4177-ba1a-999de735e682",
            read_file,
        )

        create_edge(
            local_client, parent_process_view.uid, "read_files", read_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_read_files(FileQuery().with_file_path(eq="/folder/file.txt"))
            .query_first(local_client)
        )

        assert queried_process
        assert (
            queried_process.node_key
            == "test_with_read_files-669a3693-d960-401c-8d29-5d669ffcd660"
        )

        assert queried_process.process_id == 100
        assert queried_process.process_name == "word.exe"

        assert len(queried_process.read_files) == 1
        assert (
            queried_process.read_files[0].node_key
            == "test_with_read_files-aa9248ec-36ee-4177-ba1a-999de735e682"
        )
        assert queried_process.read_files[0].file_path == "/folder/file.txt"

    def test_with_wrote_files(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        local_client = DgraphClient(DgraphClientStub("localhost:9080"))

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            local_client,
            "Process",
            ProcessView,
            "test_with_wrote_files-8f0761fb-2ffe-4d4b-ab38-68e5489f56dc",
            parent_process,
        )

        wrote_file = {
            "file_path": "/folder/file.txt",
            "created_timestamp": created_timestamp + 1000,
        }  # type: Dict[str, Property]

        wrote_file_view = upsert(
            local_client,
            "File",
            FileView,
            "test_with_wrote_files-2325c49a-95b4-423f-96d0-99539fe03833",
            wrote_file,
        )

        create_edge(
            local_client, parent_process_view.uid, "wrote_files", wrote_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_node_key(
                eq="test_with_wrote_files-8f0761fb-2ffe-4d4b-ab38-68e5489f56dc"
            )
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_wrote_files(
                FileQuery()
                .with_node_key(
                    eq="test_with_wrote_files-2325c49a-95b4-423f-96d0-99539fe03833"
                )
                .with_file_path(eq="/folder/file.txt")
            )
            .query_first(local_client)
        )

        assert queried_process
        assert (
            queried_process.node_key
            == "test_with_wrote_files-8f0761fb-2ffe-4d4b-ab38-68e5489f56dc"
        )
        assert queried_process.process_id == 100
        assert queried_process.process_name == "word.exe"

        assert len(queried_process.wrote_files) == 1
        assert (
            queried_process.wrote_files[0].node_key
            == "test_with_wrote_files-2325c49a-95b4-423f-96d0-99539fe03833"
        )
        assert queried_process.wrote_files[0].file_path == "/folder/file.txt"


if __name__ == "__main__":
    unittest.main()
