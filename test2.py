from typing import *

from grapl_analyzerlib.nodes.types import PropertyT
from grapl_analyzerlib.nodes.viewable import EdgeViewT, ForwardEdgeView
from grapl_analyzerlib.nodes.comparators import (
    Cmp,
    IntCmp,
    _int_cmps,
    StrCmp,
    _str_cmps,
)
from grapl_analyzerlib.prelude import *


from pydgraph import DgraphClient, DgraphClientStub  # type: ignore

IAuidQuery = TypeVar("IAuidQuery", bound="AuidQuery")


class AuidQuery(DynamicNodeQuery):
    def __init__(self):
        super(AuidQuery, self).__init__("Auid", AuidView)
        self._auid = []  # type: List[List[Cmp[int]]]

    def with_auid(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        self.set_int_property_filter("auid", _int_cmps("auid", eq=eq, gt=gt, lt=lt))
        return self


IAuidView = TypeVar("IAuidView", bound="AuidView")


class AuidView(DynamicNodeView):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: str,
        node_type: str,
        auid: Optional[int] = None,
    ):
        super(AuidView, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid, node_type=node_type
        )
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.node_type = node_type

        self.auid = auid

    def get_auid(self) -> Optional[int]:
        if not self.auid:
            self.auid = cast(Optional[int], self.fetch_property("auid", int))
        return self.auid

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            "auid": int,
        }

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        f_edges = {}  # type: Dict[str, Optional["EdgeViewT"]]

        return cast(
            'Mapping[str, "EdgeViewT"]',
            {fe[0]: fe[1] for fe in f_edges.items() if fe[1]},
        )

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {}  # type: Dict[str, Optional[ForwardEdgeView]]

        return cast(
            "Mapping[str, ForwardEdgeView]",
            {fe[0]: fe[1] for fe in f_edges.items() if fe[1]},
        )

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        props = {
            "auid": self.auid,
        }

        return {p[0]: p[1] for p in props.items() if p[1] is not None}


from typing import *

from grapl_analyzerlib.nodes.types import PropertyT
from grapl_analyzerlib.nodes.viewable import EdgeViewT, ForwardEdgeView
from grapl_analyzerlib.nodes.comparators import (
    Cmp,
    IntCmp,
    _int_cmps,
    StrCmp,
    _str_cmps,
)
from grapl_analyzerlib.prelude import *

from pydgraph import DgraphClient

IAuidAssumptionQuery = TypeVar("IAuidAssumptionQuery", bound="AuidAssumptionQuery")


class AuidAssumptionQuery(DynamicNodeQuery):
    def __init__(self):
        super(AuidAssumptionQuery, self).__init__("AuidAssumption", AuidAssumptionView)
        self._assumed_timestamp = []  # type: List[List[Cmp[int]]]
        self._assuming_process_id = []  # type: List[List[Cmp[int]]]

        self._assumed_auid = None  # type: Optional[IAuidQuery]
        self._assuming_process = None  # type: Optional[IProcessQuery]

    def with_assumed_timestamp(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        self.set_int_property_filter(
            "assumed_timestamp", _int_cmps("assumed_timestamp", eq=eq, gt=gt, lt=lt)
        )
        return self

    def with_assuming_process_id(
        self: "NQ",
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        self.set_int_property_filter(
            "assuming_process_id", _int_cmps("assuming_process_id", eq=eq, gt=gt, lt=lt)
        )
        return self

    def with_assumed_auid(
        self: "NQ", assumed_auid_query: Optional["IAuidQuery"] = None
    ) -> "NQ":
        assumed_auid = assumed_auid_query or AuidQuery()

        self.set_forward_edge_filter("assumed_auid", assumed_auid)
        assumed_auid.set_reverse_edge_filter("~assumed_auid", self, "assumed_auid")
        return self

    def with_assuming_process(
        self: "NQ", assuming_process_query: Optional["IProcessQuery"] = None
    ) -> "NQ":
        assuming_process = assuming_process_query or ProcessQuery()

        self.set_forward_edge_filter("assuming_process", assuming_process)
        assuming_process.set_reverse_edge_filter(
            "~assuming_process", self, "assuming_process"
        )
        return self


IAuidAssumptionView = TypeVar("IAuidAssumptionView", bound="AuidAssumptionView")


class AuidAssumptionView(DynamicNodeView):
    def __init__(
        self,
        dgraph_client: DgraphClient,
        node_key: str,
        uid: str,
        node_type: str,
        assumed_timestamp: Optional[int] = None,
        assuming_process_id: Optional[int] = None,
        assumed_auid: " Optional[AuidView]" = None,
        assuming_process: " Optional[ProcessView]" = None,
    ):
        super(AuidAssumptionView, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid, node_type=node_type
        )
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.node_type = node_type

        self.assumed_timestamp = assumed_timestamp
        self.assuming_process_id = assuming_process_id
        self.assumed_auid = assumed_auid
        self.assuming_process = assuming_process

    def get_assumed_timestamp(self) -> Optional[int]:
        if not self.assumed_timestamp:
            self.assumed_timestamp = cast(
                Optional[int], self.fetch_property("assumed_timestamp", int)
            )
        return self.assumed_timestamp

    def get_assuming_process_id(self) -> Optional[int]:
        if not self.assuming_process_id:
            self.assuming_process_id = cast(
                Optional[int], self.fetch_property("assuming_process_id", int)
            )
        return self.assuming_process_id

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {
            "assumed_timestamp": int,
            "assuming_process_id": int,
        }

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        f_edges = {
            "assumed_auid": AuidView,
            "assuming_process": ProcessView,
        }  # type: Dict[str, Optional["EdgeViewT"]]

        return cast(
            'Mapping[str, "EdgeViewT"]',
            {fe[0]: fe[1] for fe in f_edges.items() if fe[1]},
        )

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {
            "assumed_auid": self.assumed_auid,
            "assuming_process": self.assuming_process,
        }  # type: Dict[str, Optional[ForwardEdgeView]]

        return cast(
            "Mapping[str, ForwardEdgeView]",
            {fe[0]: fe[1] for fe in f_edges.items() if fe[1]},
        )

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        props = {
            "assumed_timestamp": self.assumed_timestamp,
            "assuming_process_id": self.assuming_process_id,
        }

        return {p[0]: p[1] for p in props.items() if p[1] is not None}


class AuidAssumptionExtendsAuidQuery(AuidQuery):
    def with_auid_assumptions(
        self: "NQ", auid_assumptions_query: Optional["IAuidAssumptionQuery"] = None
    ) -> "NQ":
        auid_assumptions = auid_assumptions_query or AuidAssumptionQuery()
        auid_assumptions.with_assumed_auid(cast("AuidQuery", self))

        return self


class AuidAssumptionExtendsProcessQuery(ProcessQuery):
    def with_assumed_auid(
        self: "NQ", assumed_auid_query: Optional["IAuidAssumptionQuery"] = None
    ) -> "NQ":
        assumed_auid = assumed_auid_query or AuidAssumptionQuery()
        assumed_auid.with_assuming_process(cast("ProcessQuery", self))

        return self


class AuidAssumptionExtendsAuidView(AuidView):
    def get_auid_assumptions(self,) -> "AuidAssumptionView":
        return cast(
            "AuidAssumptionView", self.fetch_edge("~assumed_auid", AuidAssumptionView)
        )


class AuidAssumptionExtendsProcessView(ProcessView):
    def get_assumed_auid(self,) -> "AuidAssumptionView":
        return cast(
            "AuidAssumptionView",
            self.fetch_edge("~assuming_process", AuidAssumptionView),
        )


if __name__ == "__main__":

    # Subclass from any other plugins that extend ProcessQuery
    # to add extensions to ProcessQuery
    class EProcessQuery(AuidAssumptionExtendsProcessQuery):
        pass

    class EProcessView(AuidAssumptionExtendsProcessView):
        pass

    # We can call all of the standard process query methods
    proc = (
        EProcessQuery()
        .with_assumed_auid()
        .with_process_id()
        .query_first(DgraphClient(DgraphClientStub("localhost:9080")))
    )

    if proc:
        parent = proc.get_parent()
        if parent:
            parent.get_process_id()
            assumed_auid = parent.get_assumed_auid()
