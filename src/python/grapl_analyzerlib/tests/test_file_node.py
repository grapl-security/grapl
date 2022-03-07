import unittest

from typing import cast, Optional

import hypothesis

import hypothesis.strategies as st

from hypothesis import given

import pytest

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.nodes.file import FileQuery, FileView
from grapl_analyzerlib.test_utils.dgraph_utils import upsert


def get_or_create_file_node(
    graph_client: GraphClient,
    node_key,
    file_path: Optional[str] = None,
    file_extension: Optional[str] = None,
    file_mime_type: Optional[str] = None,
    file_version: Optional[str] = None,
    file_description: Optional[str] = None,
    file_product: Optional[str] = None,
    file_company: Optional[str] = None,
    file_directory: Optional[str] = None,
    file_hard_links: Optional[str] = None,
    signed: Optional[str] = None,
    signed_status: Optional[str] = None,
    md5_hash: Optional[str] = None,
    sha1_hash: Optional[str] = None,
    sha256_hash: Optional[str] = None,
    file_size: Optional[str] = None,
    file_inode: Optional[int] = None,
) -> FileView:
    file = {
        "node_key": node_key,
        "file_path": file_path,
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
    }

    file = {k: v for (k, v) in file.items() if v is not None}

    return cast(FileView, upsert(graph_client, "File", FileView, node_key, file))


file_gen = {
    "node_key": st.uuids(),
    "file_path": st.text(min_size=1, max_size=64),
    "file_extension": st.text(min_size=1, max_size=64),
    "file_mime_type": st.text(min_size=1, max_size=64),
    "file_size": st.integers(min_value=0, max_value=2 ** 48),
    "file_version": st.text(min_size=1, max_size=64),
    "file_description": st.text(min_size=1, max_size=64),
    "file_product": st.text(min_size=1, max_size=64),
    "file_company": st.text(min_size=1, max_size=64),
    "file_directory": st.text(min_size=1, max_size=64),
    "file_inode": st.integers(min_value=0, max_value=2 ** 48),
    "file_hard_links": st.text(min_size=1, max_size=64),
    "signed": st.booleans(),
    "signed_status": st.text(min_size=1, max_size=64),
    "md5_hash": st.text(min_size=1, max_size=64),
    "sha1_hash": st.text(min_size=1, max_size=64),
    "sha256_hash": st.text(min_size=1, max_size=64),
}


@pytest.mark.integration_test
class TestFileQuery(unittest.TestCase):
    @hypothesis.settings(deadline=None)
    @given(**file_gen)
    def test_single_file_contains_key(
        self,
        node_key,
        file_path,
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
    ) -> None:
        node_key = "test_single_file_contains_key" + str(node_key)
        signed = "true" if signed else "false"

        graph_client = GraphClient()

        get_or_create_file_node(
            graph_client,
            node_key,
            file_path=file_path,
            file_extension=file_extension,
            file_mime_type=file_mime_type,
            file_size=file_size,
            file_version=file_version,
            file_description=file_description,
            file_product=file_product,
            file_company=file_company,
            file_directory=file_directory,
            file_inode=file_inode,
            file_hard_links=file_hard_links,
            signed=signed,
            signed_status=signed_status,
            md5_hash=md5_hash,
            sha1_hash=sha1_hash,
            sha256_hash=sha256_hash,
        )

        queried_proc = FileQuery().query_first(graph_client, contains_node_key=node_key)

        assert node_key == queried_proc.node_key

        assert file_path == queried_proc.get_file_path() or ""
        assert file_extension == queried_proc.get_file_extension() or ""
        assert file_mime_type == queried_proc.get_file_mime_type() or ""
        assert file_version == queried_proc.get_file_version() or ""
        assert file_description == queried_proc.get_file_description() or ""
        assert file_product == queried_proc.get_file_product() or ""
        assert file_company == queried_proc.get_file_company() or ""
        assert file_directory == queried_proc.get_file_directory() or ""
        assert file_hard_links == queried_proc.get_file_hard_links() or ""
        assert signed == queried_proc.get_signed() or ""
        assert signed_status == queried_proc.get_signed_status() or ""
        assert md5_hash == queried_proc.get_md5_hash() or ""
        assert sha1_hash == queried_proc.get_sha1_hash() or ""
        assert sha256_hash == queried_proc.get_sha256_hash() or ""
        assert file_size == queried_proc.get_file_size()
        assert file_inode == queried_proc.get_file_inode()

    @hypothesis.settings(deadline=None)
    @given(**file_gen)
    def test_single_file_view_parity_eq(
        self,
        node_key,
        file_path,
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
        node_key = "test_single_file_view_parity_eq" + str(node_key)
        signed = "true" if signed else "false"
        graph_client = GraphClient()

        get_or_create_file_node(
            graph_client,
            node_key,
            file_path=file_path,
            file_extension=file_extension,
            file_mime_type=file_mime_type,
            file_size=file_size,
            file_version=file_version,
            file_description=file_description,
            file_product=file_product,
            file_company=file_company,
            file_directory=file_directory,
            file_inode=file_inode,
            file_hard_links=file_hard_links,
            signed=signed,
            signed_status=signed_status,
            md5_hash=md5_hash,
            sha1_hash=sha1_hash,
            sha256_hash=sha256_hash,
        )

        queried_file = (
            FileQuery()
            .with_node_key(eq=node_key)
            .with_file_path(eq=file_path)
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
            .query_first(graph_client)
        )

        assert node_key == queried_file.node_key

        assert file_path == queried_file.get_file_path()
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
