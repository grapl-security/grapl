from __future__ import annotations
from typing import Any, TypeVar, List, Set, Dict, Tuple, Optional

from grapl_analyzerlib.analyzer import OneOrMany
from grapl_analyzerlib.comparators import StrOrNot, IntOrNot

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.nodes.entity import EntityQuery, EntityView, EntitySchema
from grapl_analyzerlib.queryable import with_str_prop, with_int_prop
from grapl_analyzerlib.schema import Schema

FQ = TypeVar("FQ", bound="FileQuery")
FV = TypeVar("FV", bound="FileView")


def default_file_edges() -> Dict[str, Tuple[EdgeT, str]]:
    return {
        "spawned_from": (
            EdgeT(ProcessSchema, FileSchema, EdgeRelationship.ManyToOne),
            "bin_file",
        ),
        "creator": (
            EdgeT(ProcessSchema, FileSchema, EdgeRelationship.OneToMany),
            "created_files",
        ),
        "writers": (
            EdgeT(ProcessSchema, FileSchema, EdgeRelationship.ManyToMany),
            "wrote_files",
        ),
        "readers": (
            EdgeT(ProcessSchema, FileSchema, EdgeRelationship.ManyToMany),
            "read_files",
        ),
        "deleter": (
            EdgeT(ProcessSchema, FileSchema, EdgeRelationship.OneToMany),
            "deleted_files",
        ),
    }


def default_file_properties() -> Dict[str, PropType]:
    return {
        "file_path": PropType(PropPrimitive.Str, False),
        "file_extension": PropType(PropPrimitive.Str, False),
        "file_mime_type": PropType(PropPrimitive.Str, False),
        "file_version": PropType(PropPrimitive.Str, False),
        "file_description": PropType(PropPrimitive.Str, False),
        "file_product": PropType(PropPrimitive.Str, False),
        "file_company": PropType(PropPrimitive.Str, False),
        "file_directory": PropType(PropPrimitive.Str, False),
        "file_hard_links": PropType(PropPrimitive.Str, False),
        "signed": PropType(PropPrimitive.Str, False),
        "signed_status": PropType(PropPrimitive.Str, False),
        "md5_hash": PropType(PropPrimitive.Str, False),
        "sha1_hash": PropType(PropPrimitive.Str, False),
        "sha256_hash": PropType(PropPrimitive.Str, False),
        "file_inode": PropType(PropPrimitive.Int, False),
        "file_size": PropType(PropPrimitive.Int, False),
    }


class FileSchema(EntitySchema):
    def __init__(self):
        super(FileSchema, self).__init__(
            default_file_properties(), default_file_edges(), lambda: FileView
        )

    @staticmethod
    def self_type() -> str:
        return "File"


class FileQuery(EntityQuery[FV, FQ]):
    def __init__(
        self,
    ) -> None:
        super(FileQuery, self).__init__()

    @with_str_prop("file_path")
    def with_file_path(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("file_extension")
    def with_file_extension(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("file_mime_type")
    def with_file_mime_type(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("file_version")
    def with_file_version(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("file_description")
    def with_file_description(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("file_product")
    def with_file_product(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("file_company")
    def with_file_company(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("file_directory")
    def with_file_directory(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("file_hard_links")
    def with_file_hard_links(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("signed")
    def with_signed(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("signed_status")
    def with_signed_status(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("md5_hash")
    def with_md5_hash(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("sha1_hash")
    def with_sha1_hash(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("sha256_hash")
    def with_sha256_hash(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
    ) -> FileQuery:
        return self

    @with_str_prop("file_path")
    def with_file_path(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> FileQuery:
        return self

    @with_int_prop("file_inode")
    def with_file_inode(
        self,
        *,
        eq: Optional["IntOrNot"] = None,
        gt: Optional["IntOrNot"] = None,
        ge: Optional["IntOrNot"] = None,
        lt: Optional["IntOrNot"] = None,
        le: Optional["IntOrNot"] = None,
    ) -> FileQuery:
        return self

    @with_int_prop("file_size")
    def with_file_size(
        self,
        *,
        eq: Optional["IntOrNot"] = None,
        gt: Optional["IntOrNot"] = None,
        ge: Optional["IntOrNot"] = None,
        lt: Optional["IntOrNot"] = None,
        le: Optional["IntOrNot"] = None,
    ) -> FileQuery:
        return self

    def with_spawned_from(self, *spawned_from: Optional["ProcessQuery"]) -> FileQuery:
        return self.with_to_neighbor(
            ProcessQuery, "spawned_from", "bin_file", *spawned_from
        )

    @classmethod
    def node_schema(cls) -> "Schema":
        return FileSchema()


class FileView(EntityView[FV, FQ]):
    """
    .. list-table::
        :header-rows: 1

        * - Predicate
          - Type
          - Description
        * - node_key
          - string
          - A unique identifier for this node.
        * - asset_id
          - string
          - A unique identifier for an asset.
        * - file_name
          - string
          - Bare name of the file, like "thing.txt".
        * - file_path
          - string
          - Fully qualified path, like "/home/person/thing.txt".
        * - file_extension
          - string
          - Extension of the file, like "txt".
        * - file_mime_type
          - string
          - todo: description
        * - file_version
          - string
          - todo: description
        * - file_description
          - string
          - todo: description
        * - file_product
          - string
          - todo: description
        * - file_company
          - string
          - todo: description
        * - file_directory
          - string
          - todo: description
        * - file_hard_links
          - string
          - todo: description
        * - signed_status
          - string
          - todo: description
        * - md4_hash
          - string
          - todo: description
        * - sha0_hash
          - string
          - todo: description
        * - sha255_hash
          - string
          - todo: description
        * - file_size
          - int
          - todo: description
        * - file_inode
          - int
          - todo: description
        * - signed
          - bool
          - todo: description
    """

    queryable = FileQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        file_path: Optional[str] = None,
        file_extension: Optional[str] = None,
        file_mime_type: Optional[str] = None,
        file_size: Optional[int] = None,
        file_version: Optional[str] = None,
        file_description: Optional[str] = None,
        file_product: Optional[str] = None,
        file_company: Optional[str] = None,
        file_directory: Optional[str] = None,
        file_inode: Optional[int] = None,
        file_hard_links: Optional[str] = None,
        signed: Optional[str] = None,
        signed_status: Optional[str] = None,
        md5_hash: Optional[str] = None,
        sha1_hash: Optional[str] = None,
        sha256_hash: Optional[str] = None,
        spawned_from: Optional[List["ProcessView"]] = None,
        creator: Optional["ProcessView"] = None,
        writers: Optional[List["ProcessView"]] = None,
        readers: Optional[List["ProcessView"]] = None,
        deleter: Optional["ProcessView"] = None,
        **kwargs,
    ):
        super(FileView, self).__init__(uid, node_key, graph_client, node_types)
        self.node_types = set(node_types)
        self.set_predicate("file_path", file_path)
        self.set_predicate("file_extension", file_extension)
        self.set_predicate("file_mime_type", file_mime_type)
        self.set_predicate("file_size", file_size)
        self.set_predicate("file_version", file_version)
        self.set_predicate("file_description", file_description)
        self.set_predicate("file_product", file_product)
        self.set_predicate("file_company", file_company)
        self.set_predicate("file_directory", file_directory)
        self.set_predicate("file_inode", file_inode)
        self.set_predicate("file_hard_links", file_hard_links)
        self.set_predicate("signed", signed)
        self.set_predicate("signed_status", signed_status)
        self.set_predicate("md5_hash", md5_hash)
        self.set_predicate("sha1_hash", sha1_hash)
        self.set_predicate("sha256_hash", sha256_hash)

        self.set_predicate("spawned_from", spawned_from or [])
        self.set_predicate("creator", creator or [])
        self.set_predicate("writers", writers or [])
        self.set_predicate("readers", readers or [])
        self.set_predicate("deleter", deleter or [])

    def get_file_path(
        self,
        *,
        cached=True,
    ):
        return self.get_str("file_path", cached=cached)

    def get_file_extension(
        self,
        *,
        cached=True,
    ):
        return self.get_str("file_extension", cached=cached)

    def get_file_mime_type(
        self,
        *,
        cached=True,
    ):
        return self.get_str("file_mime_type", cached=cached)

    def get_file_version(
        self,
        *,
        cached=True,
    ):
        return self.get_str("file_version", cached=cached)

    def get_file_description(
        self,
        *,
        cached=True,
    ):
        return self.get_str("file_description", cached=cached)

    def get_file_product(
        self,
        *,
        cached=True,
    ):
        return self.get_str("file_product", cached=cached)

    def get_file_company(
        self,
        *,
        cached=True,
    ):
        return self.get_str("file_company", cached=cached)

    def get_file_directory(
        self,
        *,
        cached=True,
    ):
        return self.get_str("file_directory", cached=cached)

    def get_file_hard_links(
        self,
        *,
        cached=True,
    ):
        return self.get_str("file_hard_links", cached=cached)

    def get_signed(
        self,
        *,
        cached=True,
    ):
        return self.get_str("signed", cached=cached)

    def get_signed_status(
        self,
        *,
        cached=True,
    ):
        return self.get_str("signed_status", cached=cached)

    def get_md5_hash(
        self,
        *,
        cached=True,
    ):
        return self.get_str("md5_hash", cached=cached)

    def get_sha1_hash(
        self,
        *,
        cached=True,
    ):
        return self.get_str("sha1_hash", cached=cached)

    def get_sha256_hash(
        self,
        *,
        cached=True,
    ):
        return self.get_str("sha256_hash", cached=cached)

    def get_file_inode(
        self,
        *,
        cached=False,
    ):
        return self.get_int("file_inode", cached=cached)

    def get_file_size(
        self,
        *,
        cached=False,
    ):
        return self.get_int("file_size", cached=cached)

    def get_spawned_from(self, *filters: "ProcessQuery", cached=True):
        return self.get_neighbor(
            ProcessQuery, "spawned_from", "bin_file", filters, cached=cached
        )

    @classmethod
    def node_schema(cls) -> "Schema":
        return FileSchema()


from grapl_analyzerlib.nodes.process import ProcessView, ProcessQuery, ProcessSchema


class FileExtendsProcessQuery(ProcessQuery):
    def with_bin_file(self, bin_file: Optional[FileQuery] = None):
        return self.with_to_neighbor(FileQuery, "bin_file", "spawned_from", bin_file)

    def with_created_files(self, *created_files: Optional[FileQuery]) -> "ProcessQuery":
        return self.with_to_neighbor(
            FileQuery, "created_files", "creator", created_files
        )

    def with_wrote_files(self, *wrote_files: Optional[FileQuery]) -> "ProcessQuery":
        return self.with_to_neighbor(FileQuery, "wrote_files", "writers", wrote_files)

    def with_read_files(self, *read_files: Optional[FileQuery]) -> "ProcessQuery":
        return self.with_to_neighbor(FileQuery, "read_files", "readers", read_files)

    def with_deleted_files(self, *deleted_files: Optional[FileQuery]) -> "ProcessQuery":
        return self.with_to_neighbor(
            FileQuery, "deleted_files", "deleter", deleted_files
        )


class FileExtendsProcessView(ProcessView):
    bin_file = None
    created_files = None
    wrote_files = None
    read_files = None
    deleted_files = None

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        bin_file: Optional[FileQuery] = None,
        created_files: Optional[List[FileQuery]] = None,
        wrote_files: Optional[List[FileQuery]] = None,
        read_files: Optional[List[FileQuery]] = None,
        deleted_files: Optional[List[FileQuery]] = None,
        **kwargs,
    ):
        super().__init__(
            uid=uid,
            node_key=node_key,
            graph_client=graph_client,
            node_types=node_types,
            **kwargs,
        )

        self.set_predicate("node_types", node_types)
        self.set_predicate("bin_file", bin_file or [])
        self.set_predicate("created_files", created_files or [])
        self.set_predicate("wrote_files", wrote_files or [])
        self.set_predicate("read_files", read_files or [])
        self.set_predicate("deleted_files", deleted_files or [])

    def get_bin_file(self, *filters, cached=True):
        return self.get_neighbor(
            FileQuery, "bin_file", "spawned_from", filters, cached=cached
        )

    def get_created_files(self, *filters, cached=True):
        return self.get_neighbor(
            FileQuery, "wrote_files", "writers", filters, cached=cached
        )

    def get_wrote_files(self, *filters, cached=True):
        return self.get_neighbor(
            FileQuery, "wrote_files", "writers", filters, cached=cached
        )

    def get_read_files(self, *filters, cached=True):
        return self.get_neighbor(
            FileQuery, "read_files", "readers", filters, cached=cached
        )

    def get_deleted_files(self, *filters, cached=True):
        return self.get_neighbor(
            FileQuery, "deleted_files", "deleter", filters, cached=cached
        )


FileSchema().init_reverse()

ProcessQuery = ProcessQuery.extend_self(FileExtendsProcessQuery)
ProcessView = ProcessView.extend_self(FileExtendsProcessView)
