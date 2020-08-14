from typing import Any, TypeVar, List, Set, Dict, Tuple, Optional

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.nodes.entity import EntityQuery, EntityView, EntitySchema
from grapl_analyzerlib.queryable import with_str_prop
from grapl_analyzerlib.schema import Schema

FQ = TypeVar("FQ", bound="FileQuery")
FV = TypeVar("FV", bound="FileView")


def default_file_edges() -> Dict[str, Tuple[EdgeT, str]]:
    return {
        "spawned_from": (
            EdgeT(FileSchema, ProcessSchema, EdgeRelationship.OneToMany),
            "bin_file",
        ),
        "creator": (
            EdgeT(FileSchema, ProcessSchema, EdgeRelationship.OneToMany),
            "created_files",
        ),
        "writers": (
            EdgeT(FileSchema, ProcessSchema, EdgeRelationship.OneToMany),
            "wrote_files",
        ),
        "readers": (
            EdgeT(FileSchema, ProcessSchema, EdgeRelationship.OneToMany),
            "read_files",
        ),
        "deleter": (
            EdgeT(FileSchema, ProcessSchema, EdgeRelationship.OneToMany),
            "deleted_files",
        ),
    }


def default_file_properties() -> Dict[str, PropType]:
    return {
        "file_path": PropType(PropPrimitive.Str, False),
        "file_extension": PropType(PropPrimitive.Str, False),
        "file_mime_type": PropType(PropPrimitive.Str, False),
        "file_size": PropType(PropPrimitive.Int, False),
        "file_version": PropType(PropPrimitive.Str, False),
        "file_description": PropType(PropPrimitive.Str, False),
        "file_product": PropType(PropPrimitive.Str, False),
        "file_company": PropType(PropPrimitive.Str, False),
        "file_directory": PropType(PropPrimitive.Str, False),
        "file_inode": PropType(PropPrimitive.Int, False),
        "file_hard_links": PropType(PropPrimitive.Str, False),
        "signed": PropType(PropPrimitive.Str, False),
        "signed_status": PropType(PropPrimitive.Str, False),
        "md5_hash": PropType(PropPrimitive.Str, False),
        "sha1_hash": PropType(PropPrimitive.Str, False),
        "sha256_hash": PropType(PropPrimitive.Str, False),
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
    def __init__(self,) -> None:
        super(FileQuery, self).__init__()

    @with_str_prop('file_path')
    def with_file_path(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> "FileQuery":
        pass

    def with_spawned_from(self, *spawned_from: Optional["ProcessQuery"]) -> "FileQuery":
        return self.with_to_neighbor(ProcessQuery, 'spawned_from', 'bin_file', *spawned_from)

    @staticmethod
    def extend_self(*types):
        for t in types:
            method_list = [
                method for method in dir(t) if method.startswith("__") is False
            ]
            for method in method_list:
                setattr(FileQuery, method, getattr(t, method))
        return type("FileQuery", types, {})

    @classmethod
    def node_schema(cls) -> "Schema":
        return FileSchema()


class FileView(EntityView[FV, FQ]):
    queryable = FileQuery

    def __init__(
        self,
        uid: str,
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
        self.file_path = file_path
        self.file_extension = file_extension
        self.file_mime_type = file_mime_type
        self.file_size = file_size
        self.file_version = file_version
        self.file_description = file_description
        self.file_product = file_product
        self.file_company = file_company
        self.file_directory = file_directory
        self.file_inode = file_inode
        self.file_hard_links = file_hard_links
        self.signed = signed
        self.signed_status = signed_status
        self.md5_hash = md5_hash
        self.sha1_hash = sha1_hash
        self.sha256_hash = sha256_hash

        self.spawned_from = spawned_from or []
        self.creator = creator or []
        self.writers = writers or []
        self.readers = readers or []
        self.deleter = deleter or []
        for key, value in kwargs.items():
            setattr(self, key, value)

    def get_spawned_from(self, *filters: 'ProcessQuery', cached=True):
        return self.get_neighbor(ProcessQuery, 'spawned_from', 'bin_file', filters, cached=cached)

    @staticmethod
    def extend_self(*types):
        for t in types:
            method_list = [
                method for method in dir(t) if method.startswith("__") is False
            ]
            for method in method_list:
                setattr(FileView, method, getattr(t, method))
        return type("FileView", types, {})

    @classmethod
    def node_schema(cls) -> "Schema":
        return FileSchema()


from grapl_analyzerlib.nodes.process import ProcessView, ProcessQuery, ProcessSchema


class FileExtendsProcessQuery(ProcessQuery):
    def with_bin_file(self, bin_file: Optional[FileQuery] = None):
        return self.with_to_neighbor(FileQuery, 'bin_file', 'spawned_from', bin_file)

    def with_created_files(self, *created_files: Optional[FileQuery]) -> "ProcessQuery":
        return self.with_to_neighbor(FileQuery, 'created_files', 'creator', created_files)

    def with_wrote_files(self, *wrote_files: Optional[FileQuery]) -> "ProcessQuery":
        return self.with_to_neighbor(FileQuery, 'wrote_files', 'writers', wrote_files)

    def with_read_files(self, *read_files: Optional[FileQuery]) -> "ProcessQuery":
        return self.with_to_neighbor(FileQuery, 'read_files', 'readers', read_files)

    def with_deleted_files(self, *deleted_files: Optional[FileQuery]) -> "ProcessQuery":
        return self.with_to_neighbor(FileQuery, 'deleted_files', 'deleter', deleted_files)


class FileExtendsProcessView(ProcessView):
    bin_file = None
    created_files = None
    wrote_files = None
    read_files = None
    deleted_files = None

    def __init__(
        self,
        uid: str,
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

        self.set_predicate('node_types', node_types)
        self.set_predicate('bin_file', bin_file or [])
        self.set_predicate('created_files', created_files or [])
        self.set_predicate('wrote_files', wrote_files or [])
        self.set_predicate('read_files', read_files or [])
        self.set_predicate('deleted_files', deleted_files or [])

    def get_bin_file(self, *filters, cached=True):
        return self.get_neighbor(FileQuery, 'bin_file', 'spawned_from', filters, cached=cached)

    def get_created_files(self, *filters, cached=True):
        return self.get_neighbor(FileQuery, 'wrote_files', 'writers', filters, cached=cached)

    def get_wrote_files(self, *filters, cached=True):
        return self.get_neighbor(FileQuery, 'wrote_files', 'writers', filters, cached=cached)

    def get_read_files(self, *filters, cached=True):
        return self.get_neighbor(FileQuery, 'read_files', 'readers', filters, cached=cached)

    def get_deleted_files(self, *filters, cached=True):
        return self.get_neighbor(FileQuery, 'deleted_files', 'deleter', filters, cached=cached)


FileSchema().init_reverse()

ProcessQuery = ProcessQuery.extend_self(FileExtendsProcessQuery)
ProcessView = ProcessView.extend_self(FileExtendsProcessView)
