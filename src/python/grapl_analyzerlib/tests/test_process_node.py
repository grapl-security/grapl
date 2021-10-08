import time
import unittest
from typing import cast, Dict

import hypothesis
import hypothesis.strategies as st
import pytest
from hypothesis import given

from grapl_analyzerlib.prelude import *
from grapl_analyzerlib.test_utils.dgraph_utils import upsert, create_edge
from grapl_analyzerlib.test_utils.strategies.asset_view_strategy import (
    asset_props_strategy,
    get_or_create_asset,
    AssetProps,
)
from grapl_analyzerlib.test_utils.strategies.misc import text_dgraph_compat
from grapl_analyzerlib.test_utils.strategies.process_view_strategy import (
    process_props_strategy,
    get_or_create_process,
    ProcessProps,
)

Property = str


def assert_equal_props(a: Viewable, b: Viewable) -> None:
    """
    NOTE: Doesn't look at edges at all.
    You may need to fetch more properties from the queried one.
    """
    for k, v in a.predicates.items():
        assert v == b.predicates[k]


def assert_equal_identity(a: Viewable, b: Viewable) -> None:
    """ Assert these nodes have the same type and uuid """
    assert a.node_key == b.node_key


def get_or_create_process_node_deprecated(
    graph_client: GraphClient,
    node_key: str,
    # properties
    process_id: str,
    arguments: str,
    created_timestamp: str,
    terminate_time: str,
    image_name: str,
    process_name: str,
) -> ProcessView:
    """
    Deprecated in favor of property_view_strategy.py
    """
    node_props: Dict[str, Property] = {
        "process_id": process_id,
        "arguments": arguments,
        "created_timestamp": created_timestamp,
        "terminate_time": terminate_time,
        "image_name": image_name,
        "process_name": process_name,
    }

    return cast(
        ProcessView, upsert(graph_client, "Process", ProcessView, node_key, node_props)
    )


common_hypo_settings = hypothesis.settings(deadline=None, max_examples=25)


@pytest.mark.integration_test
class TestProcessQuery(unittest.TestCase):
    @hypothesis.settings(parent=common_hypo_settings)
    @given(process_props=process_props_strategy())
    def test_single_process_contains_key(self, process_props: ProcessProps) -> None:
        graph_client = GraphClient()
        created_proc = get_or_create_process(self, graph_client, process_props)

        # Setup complete, do some queries

        queried_proc = ProcessQuery().query_first(
            graph_client, contains_node_key=created_proc.node_key
        )

        assert queried_proc
        assert created_proc.get_process_id() == queried_proc.get_process_id()
        assert created_proc.node_key == queried_proc.node_key
        assert "Process" == queried_proc.get_node_type()
        assert created_proc.get_arguments() == queried_proc.get_arguments()
        assert (
            created_proc.get_created_timestamp() == queried_proc.get_created_timestamp()
        )
        assert created_proc.get_terminate_time() == queried_proc.get_terminate_time()
        assert created_proc.get_image_name() == queried_proc.get_image_name()
        assert created_proc.get_process_name() == queried_proc.get_process_name()

        assert not queried_proc.get_asset()

    @hypothesis.settings(parent=common_hypo_settings)
    @given(
        asset_props=asset_props_strategy(),
        process_props=process_props_strategy(),
    )
    def test_single_process_connected_to_asset_node(
        self,
        asset_props: AssetProps,
        process_props: ProcessProps,
    ):
        graph_client = GraphClient()

        created_asset = get_or_create_asset(self, graph_client, asset_props)
        created_proc = get_or_create_process(self, graph_client, process_props)

        create_edge(
            graph_client,
            created_asset.uid,
            "asset_processes",
            created_proc.uid,
        )
        create_edge(graph_client, created_proc.uid, "process_asset", created_asset.uid)
        # Setup complete, do some queries

        queried_proc = (
            ProcessQuery()
            .with_asset(AssetQuery().with_hostname(eq=created_asset.get_hostname()))
            .query_first(graph_client, contains_node_key=created_proc.node_key)
        )
        assert queried_proc
        queried_proc._expand()
        assert_equal_props(created_proc, queried_proc)
        queried_asset = queried_proc.get_asset()
        assert_equal_identity(created_asset, queried_asset)

    # Given that the code that generates timestamps only uses unsized types we can make some
    # assumptions about the data
    @hypothesis.settings(parent=common_hypo_settings)
    @given(process_props=process_props_strategy())
    def test_process_query_view_parity(self, process_props: ProcessProps):
        graph_client = GraphClient()

        created_proc = get_or_create_process(
            self,
            graph_client,
            process_props,
        )

        queried_proc = (
            ProcessQuery()
            .with_node_key(eq=created_proc.node_key)
            .query_first(graph_client)
        )

        assert queried_proc

        assert process_props["node_key"] == queried_proc.node_key
        assert "Process" == queried_proc.get_node_type()
        assert process_props["process_id"] == queried_proc.get_process_id()
        assert process_props["arguments"] == queried_proc.get_arguments()
        assert (
            process_props["created_timestamp"] == queried_proc.get_created_timestamp()
        )
        assert None == queried_proc.get_asset()
        assert process_props["terminate_time"] == queried_proc.get_terminate_time()
        assert process_props["image_name"] == queried_proc.get_image_name()
        assert process_props["process_name"] == queried_proc.get_process_name()

    @hypothesis.settings(parent=common_hypo_settings)
    @given(
        node_key=st.uuids(),
        process_id=st.integers(min_value=1, max_value=2 ** 32),
        created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
        terminate_time=st.integers(min_value=0, max_value=2 ** 48),
        image_name=st.text(min_size=1, max_size=64),
        process_name=st.text(min_size=1, max_size=64),
        arguments=st.text(min_size=1, max_size=64),
    )
    def test_process_query_view_parity_eq(
        self,
        node_key,
        process_id,
        created_timestamp,
        terminate_time,
        image_name,
        process_name,
        arguments,
    ):
        node_key = "test_process_query_view_parity_eq" + str(node_key)
        graph_client = GraphClient()
        get_or_create_process_node_deprecated(
            graph_client,
            node_key,
            process_id,
            arguments,
            created_timestamp,
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
            .with_terminate_time(eq=terminate_time)
            .with_image_name(eq=image_name)
            .with_process_name(eq=process_name)
            .query_first(graph_client)
        )

        assert node_key == queried_proc.node_key
        assert "Process" == queried_proc.get_node_type()
        assert process_id == queried_proc.get_process_id()

        assert arguments == queried_proc.get_arguments()
        assert created_timestamp == queried_proc.get_created_timestamp()
        assert terminate_time == queried_proc.get_terminate_time()
        assert image_name == queried_proc.get_image_name()
        assert process_name == queried_proc.get_process_name()

    @hypothesis.settings(parent=common_hypo_settings)
    @given(process_props=process_props_strategy())
    def test_process_query_view_miss(self, process_props: ProcessProps) -> None:
        graph_client = GraphClient()

        created_proc = get_or_create_process(self, graph_client, process_props)

        assert (
            created_proc.process_id is not None
            and created_proc.arguments is not None
            and created_proc.created_timestamp is not None
            and created_proc.terminate_time is not None
            and created_proc.image_name is not None
            and created_proc.process_name is not None
        )
        queried_proc = (
            ProcessQuery()
            .with_node_key(eq=created_proc.node_key)
            .with_process_id(eq=Not(created_proc.process_id))
            .with_arguments(eq=Not(created_proc.arguments))
            .with_created_timestamp(eq=Not(created_proc.created_timestamp))
            .with_terminate_time(eq=Not(created_proc.terminate_time))
            .with_image_name(eq=Not(created_proc.image_name))
            .with_process_name(eq=Not(created_proc.process_name))
            .query_first(graph_client)
        )

        assert not queried_proc

    # Given that the code that generates timestamps only uses unsized types we can make some
    # assumptions about the data

    @hypothesis.settings(parent=common_hypo_settings)
    @given(
        node_key=st.uuids(),
        process_id=st.integers(min_value=1, max_value=2 ** 32),
        created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
        terminate_time=st.integers(min_value=0, max_value=2 ** 48),
        image_name=text_dgraph_compat(),
        process_name=text_dgraph_compat(),
        arguments=text_dgraph_compat(),
    )
    def test_process_query_view_parity_contains(
        self,
        node_key,
        process_id,
        created_timestamp,
        terminate_time,
        image_name,
        process_name,
        arguments,
    ):
        node_key = "test_process_query_view_parity_contains" + str(node_key)
        graph_client = GraphClient()
        get_or_create_process_node_deprecated(
            graph_client,
            node_key,
            process_id=process_id,
            arguments=arguments,
            created_timestamp=created_timestamp,
            terminate_time=terminate_time,
            image_name=image_name,
            process_name=process_name,
        )

        query = ProcessQuery().with_node_key(eq=node_key)

        # Don't fuck with newlines due to a dgraph bug
        # https://github.com/dgraph-io/dgraph/issues/4694
        for prop in [arguments, image_name, process_name]:
            hypothesis.assume(len(prop) > 3)
            hypothesis.assume("\n" not in prop)
            hypothesis.assume("\\" not in prop)

        # These fail because dgraph doesn't like the query
        # 	(regexp(process_name, /00\\//))
        query.with_arguments(contains=arguments[: len(arguments) - 1])
        query.with_image_name(contains=image_name[: len(image_name) - 1])
        query.with_process_name(contains=process_name[: len(process_name) - 1])

        queried_proc = query.query_first(graph_client)

        assert queried_proc
        assert "Process" == queried_proc.get_node_type()
        assert process_id == queried_proc.get_process_id()
        assert node_key == queried_proc.node_key
        assert arguments == queried_proc.get_arguments()
        assert created_timestamp == queried_proc.get_created_timestamp()
        assert terminate_time == queried_proc.get_terminate_time()
        assert image_name == queried_proc.get_image_name()
        assert process_name == queried_proc.get_process_name()

    def test_parent_children_edge(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        graph_client = GraphClient()

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            graph_client,
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
            graph_client,
            "Process",
            ProcessView,
            "46d2862f-cb58-4062-b35e-bb310b8d5b0d",
            child_process,
        )

        create_edge(
            graph_client,
            parent_process_view.uid,
            "children",
            child_process_view.uid,
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
            .query_first(graph_client)
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
        graph_client = GraphClient()

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            graph_client,
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
            graph_client,
            "File",
            FileView,
            "9f16e0c9-33c0-4d18-9878-ef686373570b",
            bin_file,
        )

        create_edge(
            graph_client,
            parent_process_view.uid,
            "bin_file",
            bin_file_view.uid,
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
            .query_first(graph_client)
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
        graph_client = GraphClient()

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            graph_client,
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
            graph_client,
            "File",
            FileView,
            "575f103e-1a11-4650-9f1b-5b72e44dfec3",
            created_file,
        )

        create_edge(
            graph_client,
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
            .query_first(graph_client)
        )

        assert queried_process
        assert queried_process.process_id == 100

        assert len(queried_process.created_files) == 1
        created_file = queried_process.created_files[0]
        assert created_file.file_path == "/folder/file.txt"

    def test_with_deleted_files(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        graph_client = GraphClient()

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            graph_client,
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
            graph_client,
            "File",
            FileView,
            "test_with_deleted_files8b8364ea-9b47-476b-8cf0-0f724adff10f",
            deleted_file,
        )

        create_edge(
            graph_client,
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
            .query_first(graph_client)
        )

        assert queried_process
        assert queried_process.process_id == 100

    def test_with_read_files(self) -> None:
        # Given: a process with a pid 100 & process_name word.exe,
        graph_client = GraphClient()

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            graph_client,
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
            graph_client,
            "File",
            FileView,
            "test_with_read_files-aa9248ec-36ee-4177-ba1a-999de735e682",
            read_file,
        )

        create_edge(
            graph_client,
            parent_process_view.uid,
            "read_files",
            read_file_view.uid,
        )

        queried_process = (
            ProcessQuery()
            .with_process_id(eq=100)
            .with_process_name(contains="word")
            .with_created_timestamp(eq=created_timestamp)
            .with_read_files(FileQuery().with_file_path(eq="/folder/file.txt"))
            .query_first(graph_client)
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
        graph_client = GraphClient()

        created_timestamp = int(time.time())

        parent_process = {
            "process_id": 100,
            "process_name": "word.exe",
            "created_timestamp": created_timestamp,
        }  # type: Dict[str, Property]

        parent_process_view = upsert(
            graph_client,
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
            graph_client,
            "File",
            FileView,
            "test_with_wrote_files-2325c49a-95b4-423f-96d0-99539fe03833",
            wrote_file,
        )

        create_edge(
            graph_client,
            parent_process_view.uid,
            "wrote_files",
            wrote_file_view.uid,
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
            .query_first(graph_client)
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
