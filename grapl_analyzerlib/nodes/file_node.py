from typing import Optional, TypeVar, Mapping, Tuple, Any, List, cast

# noinspection Mypy
from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.comparators import (
    Cmp,
    PropertyFilter,
    StrCmp,
    _str_cmps,
    IntCmp,
    _int_cmps,
)
from grapl_analyzerlib.nodes.queryable import Queryable, NQ
from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.nodes.viewable import (
    Viewable,
    EdgeViewT,
    ForwardEdgeView,
    ReverseEdgeView,
)

T = TypeVar("T")

IFileQuery = TypeVar("IFileQuery", bound="FileQuery")
IFileView = TypeVar("IFileView", bound="FileView")


class FileQuery(Queryable["FileView"]):
    def __init__(self,) -> None:
        super(FileQuery, self).__init__(FileView)
        self._file_path = []  # type: List[List[Cmp[str]]]
        self._asset_id = []  # type: List[List[Cmp[str]]]
        self._file_extension = []  # type: List[List[Cmp[str]]]
        self._file_mime_type = []  # type: List[List[Cmp[str]]]
        self._file_size = []  # type: List[List[Cmp[int]]]
        self._file_version = []  # type: List[List[Cmp[str]]]
        self._file_description = []  # type: List[List[Cmp[str]]]
        self._file_product = []  # type: List[List[Cmp[str]]]
        self._file_company = []  # type: List[List[Cmp[str]]]
        self._file_directory = []  # type: List[List[Cmp[str]]]
        self._file_inode = []  # type: List[List[Cmp[int]]]
        self._file_hard_links = []  # type: List[List[Cmp[str]]]
        self._signed = []  # type: List[List[Cmp[str]]]
        self._signed_status = []  # type: List[List[Cmp[str]]]
        self._md5_hash = []  # type: List[List[Cmp[str]]]
        self._sha1_hash = []  # type: List[List[Cmp[str]]]
        self._sha256_hash = []  # type: List[List[Cmp[str]]]

        self._creator = None  # type: Optional['ProcessQuery']
        self._writers = None  # type: Optional['ProcessQuery']
        self._readers = None  # type: Optional['ProcessQuery']
        self._deleter = None  # type: Optional['ProcessQuery']
        self._spawned_from = None  # type: Optional['ProcessQuery']

    def with_file_path(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_path.extend(
            _str_cmps(
                "file_path",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_asset_id(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
    ) -> "NQ":
        cast("FileQuery", self)._asset_id.extend(
            _str_cmps("asset_id", eq, contains, ends_with)
        )
        return self

    def with_file_extension(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_extension.extend(
            _str_cmps(
                "file_extension",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_file_mime_type(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_mime_type.extend(
            _str_cmps(
                "file_mime_type",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_file_size(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_size.extend(_int_cmps("file_size", eq, gt, lt))
        return self

    def with_file_version(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_version.extend(
            _str_cmps(
                "file_version",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_file_description(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_description.extend(
            _str_cmps(
                "file_description",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_file_product(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_product.extend(
            _str_cmps(
                "file_product",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_file_company(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_company.extend(
            _str_cmps(
                "file_company",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_file_directory(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_directory.extend(
            _str_cmps(
                "file_directory",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_file_inode(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_inode.extend(_int_cmps("file_inode", eq, gt, lt))
        return self

    def with_file_hard_links(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
    ) -> "NQ":
        cast("FileQuery", self)._file_hard_links.extend(
            _str_cmps("file_hard_links", eq, contains, ends_with)
        )
        return self

    def with_signed(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
        starts_with: Optional["StrCmp"] = None,
        regexp: Optional["StrCmp"] = None,
        distance: Optional[Tuple["StrCmp", int]] = None,
    ) -> "NQ":
        cast("FileQuery", self)._signed.extend(
            _str_cmps(
                "signed",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )
        return self

    def with_signed_status(
        self: "NQ",
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
    ) -> "NQ":
        cast("FileQuery", self)._signed_status.extend(
            _str_cmps("signed_status", eq, contains, ends_with)
        )
        return self

    def with_md5_hash(self: "NQ", eq: Optional["StrCmp"] = None) -> "NQ":
        cast("FileQuery", self)._md5_hash.extend(_str_cmps("md5_hash", eq))
        return self

    def with_sha1_hash(self: "NQ", eq: Optional["StrCmp"] = None) -> "NQ":
        cast("FileQuery", self)._sha1_hash.extend(_str_cmps("sha1_hash", eq))
        return self

    def with_sha256_hash(self: "NQ", eq: Optional["StrCmp"] = None) -> "NQ":
        cast("FileQuery", self)._sha256_hash.extend(_str_cmps("sha256_hash", eq))
        return self

    def with_spawned_from(
        self: "NQ", spawned_from_query: Optional["ProcessQuery"] = None
    ) -> "NQ":
        spawned_from = spawned_from_query or ProcessQuery()  # type: ProcessQuery

        spawned_from._bin_file = cast("FileQuery", self)
        cast("FileQuery", self)._spawned_from = spawned_from
        return self

    def with_creator(
        self: "NQ", creator_query: Optional["ProcessQuery"] = None
    ) -> "NQ":
        creator = creator_query or ProcessQuery()  # type: ProcessQuery
        creator._created_files = cast("FileQuery", self)
        cast("FileQuery", self)._creator = creator
        return self

    def with_readers(self: "NQ", reader_query: Optional["ProcessQuery"] = None) -> "NQ":
        reader = reader_query or ProcessQuery()  # type: ProcessQuery
        reader._read_files = cast("FileQuery", self)
        cast("FileQuery", self)._readers = reader
        return self

    def _get_unique_predicate(self) -> Optional[Tuple[str, "PropertyT"]]:
        return "file_path", str

    def _get_node_type_name(self) -> str:
        return "File"

    def _get_property_filters(self) -> Mapping[str, "PropertyFilter[Property]"]:
        _prop_filters = {
            "file_path": self._file_path,
            "asset_id": self._asset_id,
            "file_extension": self._file_extension,
            "file_mime_type": self._file_mime_type,
            "file_size": self._file_size,
            "file_version": self._file_version,
            "file_description": self._file_description,
            "file_product": self._file_product,
            "file_company": self._file_company,
            "file_directory": self._file_directory,
            "file_inode": self._file_inode,
            "file_hard_links": self._file_hard_links,
            "signed": self._signed,
            "signed_status": self._signed_status,
            "md5_hash": self._md5_hash,
            "sha1_hash": self._sha1_hash,
            "sha256_hash": self._sha256_hash,
        }

        prop_filters = {p[0]: p[1] for p in _prop_filters.items() if p[1]}
        return cast("Mapping[str, PropertyFilter[Property]]", prop_filters)

    def _get_forward_edges(self) -> Mapping[str, "Queryable"]:
        return {}

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        reverse_edges = {
            "~created_files": (self._creator, "creator"),
            "~wrote_files": (self._writers, "writers"),
            "~read_files": (self._readers, "readers"),
            "~deleted_files": (self._deleter, "deleter"),
            "~bin_file": (self._spawned_from, "spawned_from"),
        }

        filtered = {
            re[0]: re[1] for re in reverse_edges.items() if re[1][0] is not None
        }

        return cast("Mapping[str, Tuple[Queryable, str]]", filtered)


class FileView(Viewable):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: str,
        node_type: Optional[str] = None,
        file_path: Optional[str] = None,
        asset_id: Optional[str] = None,
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
        creator: Optional["ProcessView"] = None,
        writers: Optional[List["ProcessView"]] = None,
        readers: Optional[List["ProcessView"]] = None,
        deleter: Optional["ProcessView"] = None,
        spawned_from: Optional[List["ProcessView"]] = None,
    ) -> None:
        super(FileView, self).__init__(dgraph_client, node_key=node_key, uid=uid)

        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.node_type = node_type
        self.file_path = file_path
        self.asset_id = asset_id
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
        if not signed:
            self.signed = None
        elif signed == "True":
            self.signed = True
        elif signed == "False":
            self.signed = False
        else:
            raise ValueError(f"signed must be True or False: {signed}")

        self.signed_status = signed_status
        self.md5_hash = md5_hash
        self.sha1_hash = sha1_hash
        self.sha256_hash = sha256_hash

        self.creator = creator
        self.writers = writers or []
        self.readers = readers or []
        self.deleter = deleter
        self.spawned_from = spawned_from or []

    def get_node_type(self) -> str:
        return 'File'

    def get_file_path(self) -> Optional[str]:
        if self.file_path is not None:
            return self.file_path
        self.file_path = cast(str, self.fetch_property("file_path", str))
        return self.file_path

    def get_asset_id(self) -> Optional[str]:
        if self.asset_id is not None:
            return self.asset_id
        self.asset_id = cast(str, self.fetch_property("asset_id", str))
        return self.asset_id

    def get_file_extension(self) -> Optional[str]:
        if self.file_extension is not None:
            return self.file_extension
        self.file_extension = cast(str, self.fetch_property("file_extension", str))
        return self.file_extension

    def get_file_mime_type(self) -> Optional[str]:
        if self.file_mime_type is not None:
            return self.file_mime_type
        self.file_mime_type = cast(str, self.fetch_property("file_mime_type", str))
        return self.file_mime_type

    def get_file_size(self) -> Optional[int]:
        if self.file_size is not None:
            return self.file_size
        self.file_size = cast(int, self.fetch_property("file_size", int))
        return self.file_size

    def get_file_version(self) -> Optional[str]:
        if self.file_version is not None:
            return self.file_version
        self.file_version = cast(str, self.fetch_property("file_version", str))
        return self.file_version

    def get_file_description(self) -> Optional[str]:
        if self.file_description is not None:
            return self.file_description
        self.file_description = cast(str, self.fetch_property("file_description", str))
        return self.file_description

    def get_file_product(self) -> Optional[str]:
        if self.file_product is not None:
            return self.file_product
        self.file_product = cast(str, self.fetch_property("file_product", str))
        return self.file_product

    def get_file_company(self) -> Optional[str]:
        if self.file_company is not None:
            return self.file_company
        self.file_company = cast(str, self.fetch_property("file_company", str))
        return self.file_company

    def get_file_directory(self) -> Optional[str]:
        if self.file_directory is not None:
            return self.file_directory
        self.file_directory = cast(str, self.fetch_property("file_directory", str))
        return self.file_directory

    def get_file_inode(self) -> Optional[int]:
        if self.file_inode is not None:
            return self.file_inode
        self.file_inode = cast(int, self.fetch_property("file_inode", int))
        return self.file_inode

    def get_file_hard_links(self) -> Optional[str]:
        if self.file_hard_links is not None:
            return self.file_hard_links
        self.file_hard_links = cast(str, self.fetch_property("file_hard_links", str))
        return self.file_hard_links

    def get_signed(self) -> Optional[bool]:
        if self.signed is not None:
            return self.signed
        signed = cast(str, self.fetch_property("signed", str))
        if not signed:
            return None

        if signed == "True":
            self.signed = True
        elif signed == "False":
            self.signed = False
        else:
            raise ValueError(f"signed must be True or False: {signed}")

        return self.signed

    def get_signed_status(self) -> Optional[str]:
        if self.signed_status is not None:
            return self.signed_status
        self.signed_status = cast(str, self.fetch_property("signed_status", str))
        return self.signed_status

    def get_md5_hash(self) -> Optional[str]:
        if self.md5_hash is not None:
            return self.md5_hash
        self.md5_hash = cast(str, self.fetch_property("md5_hash", str))
        return self.md5_hash

    def get_sha1_hash(self) -> Optional[str]:
        if self.sha1_hash is not None:
            return self.sha1_hash
        self.sha1_hash = cast(str, self.fetch_property("sha1_hash", str))
        return self.sha1_hash

    def get_sha256_hash(self) -> Optional[str]:
        if self.sha256_hash is not None:
            return self.sha256_hash
        self.sha256_hash = cast(str, self.fetch_property("sha256_hash", str))
        return self.sha256_hash

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            "file_path": str,
            "asset_id": str,
            "file_extension": str,
            "file_mime_type": str,
            "file_size": int,
            "file_version": str,
            "file_description": str,
            "file_product": str,
            "file_company": str,
            "file_directory": str,
            "file_inode": int,
            "file_hard_links": str,
            "signed": str,
            "signed_status": str,
            "md5_hash": str,
            "sha1_hash": str,
            "sha256_hash": str,
        }

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        return {}

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        return {
            "~created_files": (ProcessView, "creator"),
            "~wrote_files": ([ProcessView], "writers"),
            "~read_files": ([ProcessView], "readers"),
            "~deleted_files": (ProcessView, "deleter"),
        }

    def _get_properties(self) -> Mapping[str, "Property"]:
        props = {
            "node_key": self.node_key,
            "uid": self.uid,
            "file_path": self.file_path,
            "asset_id": self.asset_id,
            "file_extension": self.file_extension,
            "file_mime_type": self.file_mime_type,
            "file_size": self.file_size,
            "file_version": self.file_version,
            "file_description": self.file_description,
            "file_product": self.file_product,
            "file_company": self.file_company,
            "file_directory": self.file_directory,
            "file_inode": self.file_inode,
            "file_hard_links": self.file_hard_links,
            "signed": self.signed,
            "signed_status": self.signed_status,
            "md5_hash": self.md5_hash,
            "sha1_hash": self.sha1_hash,
            "sha256_hash": self.sha256_hash,
        }

        return {p[0]: p[1] for p in props.items() if p[1] is not None}

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        return dict()

    def _get_reverse_edges(self) -> "Mapping[str,  ReverseEdgeView]":
        reverse_edges = {
            "~created_files": (self.creator, "creator"),
            "~wrote_files": (self.writers, "writers"),
            "~readers": (self.readers, "readers"),
            "~deleted_files": (self.deleter, "deleter"),
        }

        filtered = {
            re[0]: re[1] for re in reverse_edges.items() if re[1][0] is not None
        }

        return cast("Mapping[str,  ReverseEdgeView]", filtered)


from grapl_analyzerlib.nodes.process_node import ProcessQuery, ProcessView
