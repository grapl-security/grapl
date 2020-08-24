from typing import *

from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.comparators import Cmp, StrCmp, _str_cmps, PropertyFilter
from grapl_analyzerlib.nodes.queryable import Queryable, NQ
from grapl_analyzerlib.nodes.types import PropertyT, Property
from grapl_analyzerlib.nodes.viewable import (
    EdgeViewT,
    ForwardEdgeView,
    Viewable,
    ReverseEdgeView,
)

IAssetQuery = TypeVar("IAssetQuery", bound="AssetQuery")
IAssetView = TypeVar("IAssetView", bound="AssetView")


class AssetQuery(Queryable["AssetView"]):
    def __init__(self) -> None:
        super(AssetQuery, self).__init__(AssetView)

        self._hostname = []  # type: List[List[Cmp[str]]]

        self._asset_processes = None  # type: Optional[IProcessQuery]
        self._files_on_asset = None  # type: Optional[IFileQuery]

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable", str]]:
        return {}

    def _get_unique_predicate(self) -> Optional[Tuple[str, "PropertyT"]]:
        return None

    def _get_node_type_name(self) -> str:
        return "Asset"

    def _get_property_filters(self) -> Mapping[str, "PropertyFilter[Property]"]:
        props = {
            "hostname": self._hostname,
        }

        return {p[0]: p[1] for p in props.items() if p[1]}

    def _get_forward_edges(self) -> Mapping[str, "Queryable[Viewable]"]:
        f_edges = {
            "asset_processes": self._asset_processes,
            "files_on_asset": self._files_on_asset,
        }

        # This is right, Mypy just doesn't recognize it as such
        return {k: v for k, v in f_edges.items() if v is not None}

    def with_processes(
        self: "NQ", process_query: Optional["IProcessQuery"] = None
    ) -> "NQ":
        process = process_query or ProcessQuery()  # type: ProcessQuery
        process._process_asset = cast(AssetQuery, self)
        cast(AssetQuery, self)._asset_processes = process
        return self

    def with_hostname(
        self: "NQ",
        eq: Optional[StrCmp] = None,
        contains: Optional[StrCmp] = None,
        ends_with: Optional[StrCmp] = None,
        starts_with: Optional[StrCmp] = None,
        regexp: Optional[StrCmp] = None,
        distance: Optional[Tuple[StrCmp, int]] = None,
    ) -> "NQ":
        self._hostname.extend(
            _str_cmps(
                "hostname",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance=distance,
            )
        )

        return self


class AssetView(Viewable):
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

    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: str,
        node_type: Optional[str] = None,
        hostname: Optional[str] = None,
        asset_processes: Optional[List["ProcessView"]] = None,
        **kwargs,
    ):
        super(AssetView, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid
        )
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid

        self.hostname = hostname

        self.asset_processes = asset_processes
        self.kwargs = kwargs

    def get_node_type(self) -> str:
        return "Asset"

    def get_hostname(self) -> Optional[str]:
        if not self.hostname:
            self.hostname = cast(Optional[str], self.fetch_property("hostname", str))
        return self.hostname

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            "hostname": str,
        }

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        return {}

    def _get_reverse_edges(self) -> "Mapping[str,  ReverseEdgeView]":
        return {}

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        f_edges = {}  # type: Dict[str, Optional["EdgeViewT"]]

        return cast(
            Mapping[str, "EdgeViewT"], {fe[0]: fe[1] for fe in f_edges.items() if fe[1]}
        )

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {}  # type: Dict[str, Optional[ForwardEdgeView]]

        return cast(
            "Mapping[str, ForwardEdgeView]",
            {fe[0]: fe[1] for fe in f_edges.items() if fe[1]},
        )

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        props = {
            "hostname": self.hostname,
        }

        return {p[0]: p[1] for p in props.items() if p[1] is not None}


from grapl_analyzerlib.nodes.process_node import (
    IProcessQuery,
    IProcessView,
    ProcessQuery,
)
from grapl_analyzerlib.nodes.file_node import IFileQuery, IFileView
from grapl_analyzerlib.nodes.process_node import ProcessView
