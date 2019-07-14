from copy import deepcopy
from typing import List, Optional, Any, Dict, Tuple

from pydgraph import DgraphClient

import grapl_analyzerlib.external_ip_node as external_ip_node
from grapl_analyzerlib.node_types import PQ, EIPQ, OCQ, EIPV, OCV
from grapl_analyzerlib.querying import Has, Cmp, Queryable, Viewable, PropertyFilter


class OutboundConnectionQuery(Queryable):
    def get_unique_predicate(self) -> Optional[str]:
        return 'port'

    def get_node_type_name(self) -> Optional[str]:
        return None

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        return [[self._uid]]

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = (
            ("node_key", self.get_node_key_filter()),
            ("uid", self.get_uid_filter()),
            ("create_time", self._create_time) if self._create_time else None,
            ("terminate_time", self._terminate_time) if self._terminate_time else None,
            ("last_seen_time", self._last_seen_time) if self._last_seen_time else None,
            ("ip", self._ip) if self._ip else None,
            ("port", self._port) if self._port else None,
        )

        return [p for p in properties if p]

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        pass

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        pass

    def __init__(self) -> None:
        self._node_key = Has(
            "node_key"
        )  # type: Cmp
        self._uid = Has(
            "uid"
        )  # type: Cmp

        self._create_time = []  # type: List[List[Cmp]]
        self._terminate_time = []  # type: List[List[Cmp]]
        self._last_seen_time = []  # type: List[List[Cmp]]
        self._ip = []  # type: List[List[Cmp]]
        self._port = []  # type: List[List[Cmp]]

        # self._internal_connection = None  # type: Optional[Any]
        self._external_connection = None  # type: Optional[EIPQ]
        self._connecting_process = None  # type: Optional[PQ]

    def with_external_connection(
            self,
            external_ip: EIPQ
    ) -> OCQ:
        external_ip = deepcopy(external_ip)
        external_ip._connections_from = self
        self._external_connection = external_ip
        return self

    def with_connecting_process(
            self,
            process: PQ
    ) -> OCQ:
        process = deepcopy(process)
        process._created_connection = self
        self._connecting_process = process
        return self


class OutboundConnectionView(Viewable):
    def __init__(self,
                 dgraph_client: DgraphClient,
                 node_key: str,
                 uid: Optional[str] = None,
                 port: Optional[str] = None,
                 external_connections: Optional[EIPV] = None,
                 ) -> None:
        super(OutboundConnectionView, self).__init__(self)

        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.uid = uid
        self.port = port

        self.external_connections = external_connections

    @staticmethod
    def from_dict(dgraph_client: DgraphClient, d: Dict[str, Any]) -> OCV:
        raw_external_connection = d.get('external_connection', None)

        external_connection = None  # type: Optional[EIPV]
        if raw_external_connection:
            external_connection = external_ip_node.ExternalIpView.from_dict(dgraph_client, raw_external_connection[0])

        return OutboundConnectionView(
            dgraph_client=dgraph_client,
            node_key=d['node_key'],
            uid=d['uid'],
            port=d.get('port'),
            external_connections=external_connection,
        )


