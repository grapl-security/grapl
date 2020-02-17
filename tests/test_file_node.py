import time
import unittest
import json
from copy import deepcopy
from typing import *

from hypothesis import given
import hypothesis.strategies as st
from pydgraph import DgraphClient, DgraphClientStub
from grapl_analyzerlib.nodes.comparators import IntCmp, _str_cmps, StrCmp, _int_cmps, escape_dgraph_str, Not
from grapl_analyzerlib.nodes.dynamic_node import DynamicNodeQuery, DynamicNodeView
from grapl_analyzerlib.nodes.process_node import ProcessQuery, ProcessView, IProcessView
from grapl_analyzerlib.nodes.file_node import FileQuery, FileView
from grapl_analyzerlib.nodes.types import Property, PropertyT
from grapl_analyzerlib.nodes.viewable import Viewable, EdgeViewT, ForwardEdgeView

from grapl_provision import provision, drop_all

def _upsert(client: DgraphClient, node_dict: Dict[str, Property]) -> str:
    if node_dict.get('uid'):
        node_dict.pop('uid')
    node_dict['uid'] = '_:blank-0'
    node_key = node_dict['node_key']
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}"))
            {{
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
    for key, value in node_props.items():
        if isinstance(value, str):
            node_props[key] = escape_dgraph_str(value)
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


def get_or_create_file_node(
        local_client: DgraphClient,
        node_key,
        file_path,
        asset_id,
        file_extension,
        file_mime_type,
        file_size,
        file_version,
        file_description,
        file_product,
        file_company,
        file_directory,
        file_inode,
        file_hard_links,
        signed,
        signed_status,
        md5_hash,
        sha1_hash,
        sha256_hash,
) -> FileView:
    file = {
        "node_key": node_key,
        "file_path": file_path,
        "asset_id": asset_id,
        "file_extension": file_extension,
        "file_mime_type": file_mime_type,
        "file_size": file_size,
        "file_version": file_version,
        "file_description": file_description,
        "file_product": file_product,
        "file_company": file_company,
        "file_directory": file_directory,
        "file_inode": file_inode,
        "file_hard_links": file_hard_links,
        "signed": signed,
        "signed_status": signed_status,
        "md5_hash": md5_hash,
        "sha1_hash": sha1_hash,
        "sha256_hash": sha256_hash,
    }  # type: Dict[str, Property]

    return cast(
        FileView,
        upsert(
            local_client,
            'File',
            FileView,
            node_key,
            file
        )
    )  # type: FileView


class TestFileQuery(unittest.TestCase):
    #
    # @classmethod
    # def setUpClass(cls):
    #     local_client = DgraphClient(DgraphClientStub('localhost:9080'))
    #
    #     # drop_all(local_client)
    #     # time.sleep(3)
    #     # provision()
    #     # provision()

    # @classmethod
    # def tearDownClass(cls):
    #     local_client = DgraphClient(DgraphClientStub('localhost:9080'))
    #
    #     drop_all(local_client)
    #     provision()

    @given(
        node_key=st.uuids(),
        file_path=st.text(),
        asset_id=st.text(),
        file_extension=st.text(),
        file_mime_type=st.text(),
        file_size=st.integers(min_value=0, max_value=2**48),
        file_version=st.text(),
        file_description=st.text(),
        file_product=st.text(),
        file_company=st.text(),
        file_directory=st.text(),
        file_inode=st.integers(min_value=0, max_value=2**48),
        file_hard_links=st.text(),
        signed=st.booleans(),
        signed_status=st.text(),
        md5_hash=st.text(),
        sha1_hash=st.text(),
        sha256_hash=st.text(),
    )
    def test_single_file_contains_key(
            self,
            node_key,
            file_path,
            asset_id,
            file_extension,
            file_mime_type,
            file_size,
            file_version,
            file_description,
            file_product,
            file_company,
            file_directory,
            file_inode,
            file_hard_links,
            signed,
            signed_status,
            md5_hash,
            sha1_hash,
            sha256_hash,
    ):
        node_key = 'test_single_file_contains_key' + str(node_key)
        local_client = DgraphClient(DgraphClientStub('localhost:9080'))

        get_or_create_file_node(
            local_client,
            node_key,
            file_path,
            asset_id,
            file_extension,
            file_mime_type,
            file_size,
            file_version,
            file_description,
            file_product,
            file_company,
            file_directory,
            file_inode,
            file_hard_links,
            signed,
            signed_status,
            md5_hash,
            sha1_hash,
            sha256_hash,
        )

        queried_proc = (
            FileQuery()
            .query_first(local_client, contains_node_key=node_key)
        )

        # assert process_view.process_id == queried_proc.get_process_id()
        assert node_key == queried_proc.node_key

        assert file_path == queried_proc.get_file_path()
        assert asset_id == queried_proc.get_asset_id()
        assert file_extension == queried_proc.get_file_extension()
        assert file_mime_type == queried_proc.get_file_mime_type()
        assert file_size == queried_proc.get_file_size()
        assert file_version == queried_proc.get_file_version()
        assert file_description == queried_proc.get_file_description()
        assert file_product == queried_proc.get_file_product()
        assert file_company == queried_proc.get_file_company()
        assert file_directory == queried_proc.get_file_directory()
        assert file_inode == queried_proc.get_file_inode()
        assert file_hard_links == queried_proc.get_file_hard_links()
        assert signed == queried_proc.get_signed()
        assert signed_status == queried_proc.get_signed_status()
        assert md5_hash == queried_proc.get_md5_hash()
        assert sha1_hash == queried_proc.get_sha1_hash()
        assert sha256_hash == queried_proc.get_sha256_hash()

    @given(
        node_key=st.uuids(),
        file_path=st.text(),
        asset_id=st.text(),
        file_extension=st.text(),
        file_mime_type=st.text(),
        file_size=st.integers(min_value=0, max_value=2**48),
        file_version=st.text(),
        file_description=st.text(),
        file_product=st.text(),
        file_company=st.text(),
        file_directory=st.text(),
        file_inode=st.integers(min_value=0, max_value=2**48),
        file_hard_links=st.text(),
        signed=st.booleans(),
        signed_status=st.text(),
        md5_hash=st.text(),
        sha1_hash=st.text(),
        sha256_hash=st.text(),
    )
    def test_single_file_view_parity_eq(
            self,
            node_key,
            file_path,
            asset_id,
            file_extension,
            file_mime_type,
            file_size,
            file_version,
            file_description,
            file_product,
            file_company,
            file_directory,
            file_inode,
            file_hard_links,
            signed,
            signed_status,
            md5_hash,
            sha1_hash,
            sha256_hash,
    ):
        node_key = 'test_single_file_view_parity_eq' + str(node_key)
        local_client = DgraphClient(DgraphClientStub('localhost:9080'))

        get_or_create_file_node(
            local_client,
            node_key,
            file_path,
            asset_id,
            file_extension,
            file_mime_type,
            file_size,
            file_version,
            file_description,
            file_product,
            file_company,
            file_directory,
            file_inode,
            file_hard_links,
            signed,
            signed_status,
            md5_hash,
            sha1_hash,
            sha256_hash,
        )

        queried_file = (
            FileQuery()
            .with_node_key(eq=node_key)
            .with_file_path(eq=file_path)
            .with_asset_id(eq=asset_id)
            .with_file_extension(eq=file_extension)
            .with_file_mime_type(eq=file_mime_type)
            .with_file_size(eq=file_size)
            .with_file_version(eq=file_version)
            .with_file_description(eq=file_description)
            .with_file_product(eq=file_product)
            .with_file_company(eq=file_company)
            .with_file_directory(eq=file_directory)
            .with_file_inode(eq=file_inode)
            .with_file_hard_links(eq=file_hard_links)
            .with_signed(eq=signed)
            .with_signed_status(eq=signed_status)
            .with_md5_hash(eq=md5_hash)
            .with_sha1_hash(eq=sha1_hash)
            .with_sha256_hash(eq=sha256_hash)
            .query_first(local_client)
        )

        assert node_key == queried_file.node_key

        assert file_path == queried_file.get_file_path()
        assert asset_id == queried_file.get_asset_id()
        assert file_extension == queried_file.get_file_extension()
        assert file_mime_type == queried_file.get_file_mime_type()
        assert file_size == queried_file.get_file_size()
        assert file_version == queried_file.get_file_version()
        assert file_description == queried_file.get_file_description()
        assert file_product == queried_file.get_file_product()
        assert file_company == queried_file.get_file_company()
        assert file_directory == queried_file.get_file_directory()
        assert file_inode == queried_file.get_file_inode()
        assert file_hard_links == queried_file.get_file_hard_links()
        assert signed == queried_file.get_signed()
        assert signed_status == queried_file.get_signed_status()
        assert md5_hash == queried_file.get_md5_hash()
        assert sha1_hash == queried_file.get_sha1_hash()
        assert sha256_hash == queried_file.get_sha256_hash()


if __name__ == '__main__':
    unittest.main()
