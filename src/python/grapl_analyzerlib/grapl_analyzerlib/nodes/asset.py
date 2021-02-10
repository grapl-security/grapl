from __future__ import annotations
from typing import Any, TypeVar, List, Set, Dict, Tuple, Optional, Union

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.queryable import Queryable
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.nodes.base import BaseView
from grapl_analyzerlib.nodes.entity import EntitySchema, EntityQuery, EntityView

AQ = TypeVar("AQ", bound="AssetQuery")
AV = TypeVar("AV", bound="AssetView")

T = TypeVar("T")

OneOrMany = Union[List[T], T]


def default_asset_properties() -> Dict[str, PropType]:
    return {
        "hostname": PropType(
            PropPrimitive.Str,
            False,
        ),
    }


def default_asset_edges() -> Dict[str, Tuple[EdgeT, str]]:
    return {
        "asset_ip": (
            EdgeT(AssetSchema, IpAddressSchema, EdgeRelationship.ManyToMany),
            "ip_assigned_to",
        ),
        "asset_processes": (
            EdgeT(
                AssetSchema,
                ProcessSchema,
                EdgeRelationship.ManyToOne,
            ),
            "process_asset",
        ),
        "files_on_asset": (
            EdgeT(
                AssetSchema,
                FileSchema,
                EdgeRelationship.ManyToOne,
            ),
            "file_asset",
        ),
    }


class AssetSchema(EntitySchema):
    def __init__(self):
        super(AssetSchema, self).__init__(
            default_asset_properties(), default_asset_edges(), view=lambda: AssetView
        )

    @staticmethod
    def self_type() -> str:
        return "Asset"


class AssetQuery(EntityQuery[AV, AQ]):
    @classmethod
    def node_schema(cls) -> Schema:
        return AssetSchema()

    def with_hostname(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> AssetQuery:
        self._property_filters["hostname"].extend(
            _str_cmps(
                predicate="hostname",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self

    def with_asset_ip(self, *asset_ips: "IpAddressQuery"):
        asset_ips = asset_ips or [IpAddressSchema()]
        self.set_neighbor_filters("asset_ip", [asset_ips])
        for asset_ip in asset_ips:
            asset_ip.set_neighbor_filters("ip_assigned_to", [self])
        return self

    def with_asset_processes(self, *asset_processes: "ProcessQuery"):
        asset_processes = asset_processes or [ProcessSchema()]
        self.set_neighbor_filters("asset_processes", [asset_processes])
        for asset_process in asset_processes:
            asset_process.set_neighbor_filters("process_asset", [self])
        return self

    def with_files_on_asset(self, *files_on_asset: "FileQuery"):
        files_on_asset = files_on_asset or [FileSchema()]
        self.set_neighbor_filters("files_on_asset", [files_on_asset])
        for file_on_asset in files_on_asset:
            file_on_asset.set_neighbor_filters("file_asset", [self])
        return self


class AssetView(EntityView[AV, AQ]):
    """
    .. list-table::
        :header-rows: 1

        * - Predicate
          - Type
          - Description
        * - node_key
          - string
          - A unique identifier for this node.
        * - hostname
          - string
          - The hostname of this asset.
        * - asset_processes
          - List[:doc:`/nodes/process`]
          - Processes associated with this asset.
    """

    queryable = AssetQuery

    @classmethod
    def node_schema(cls) -> Schema:
        return AssetSchema()

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        hostname: Optional[str] = None,
        asset_ip: Optional[List["IpAddressView"]] = None,
        asset_processes: Optional[List["ProcessView"]] = None,
        files_on_asset: Optional[List["FileView"]] = None,
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, node_types=node_types, **kwargs)
        self.set_predicate("node_types", node_types)
        self.set_predicate("hostname", hostname)
        self.set_predicate("asset_ip", asset_ip)
        self.set_predicate("asset_processes", asset_processes)
        self.set_predicate("files_on_asset", files_on_asset)

    def get_hostname(self, cached=True) -> Optional[str]:
        return self.get_str("hostname", cached=cached)

    def with_asset_ip(self, *asset_ips, cached=True):
        if cached and self.asset_ip:
            return self.asset_ip

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_asset_ip(asset_ips)
            .query_first(self.graph_client)
        )

        if self_node:
            self.asset_ip = self_node.asset_ip
        return self.asset_ip

    def with_asset_processes(self, *processes, cached=True):
        if cached and self.asset_processes:
            return self.asset_processes

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_asset_processes(processes)
            .query_first(self.graph_client)
        )

        if self_node:
            self.asset_processes = self_node.asset_processes
        return self.asset_processes

    def with_files_on_asset(self, *files, cached=True):
        if cached and self.files_on_asset:
            return self.files_on_asset

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_files_on_asset(files)
            .query_first(self.graph_client)
        )

        if self_node:
            self.files_on_asset = self_node.files_on_asset
        return self.files_on_asset


from grapl_analyzerlib.comparators import StrOrNot, _str_cmps
from grapl_analyzerlib.nodes.ip_address import (
    IpAddressSchema,
    IpAddressView,
    IpAddressQuery,
)
from grapl_analyzerlib.nodes.file import FileSchema, FileView, FileQuery
from grapl_analyzerlib.nodes.process import ProcessSchema, ProcessView, ProcessQuery

AssetSchema().init_reverse()


class AssetExtendsProcessQuery(ProcessQuery):
    def with_asset(self, *filters):
        return self.with_to_neighbor(
            AssetQuery, "process_asset", "asset_processes", filters
        )


class AssetExtendsProcessView(ProcessView):
    def get_asset(self, *filters, cached=True):
        return self.get_neighbor(
            AssetQuery, "process_asset", "asset_processes", filters, cached=cached
        )


ProcessQuery = ProcessQuery.extend_self(AssetExtendsProcessQuery)
ProcessView = ProcessView.extend_self(AssetExtendsProcessView)
