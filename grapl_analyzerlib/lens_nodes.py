import json
from collections import Mapping
from typing import Any, Dict, List, Tuple, Optional, Union, Type, Callable

from pydgraph import DgraphClient, Txn

from grapl_analyzerlib.entities import NodeView, ProcessView, PV
from grapl_analyzerlib.querying import Queryable, Viewable, PropertyFilter, V


def mut_from_response(res):
    if isinstance(res, dict):
        for element in res.values():
            if type(element) is list:
                mut_from_response(element)
            if type(element) is dict:
                element.pop('uid', None)
                mut_from_response(element)
    else:
        for element in res:
            if type(element) is list:
                mut_from_response(element)
            if type(element) is dict:
                element.pop('uid', None)
                mut_from_response(element)


class CopyingTransaction(Txn):
    def __init__(self, copying_client, read_only=False, best_effort=False) -> None:
        super().__init__(copying_client.src_client, read_only, best_effort)
        self.src_client = copying_client.src_client
        self.dst_client = copying_client.dst_client

    def query(self, query, variables=None, timeout=None, metadata=None,
              credentials=None):
        """
        Query the dst graph.
        if response, return response
        If it does not, check if it exists in src graph
        if it does, copy from src graph to dst graph
        return query on dst graph
        :return:
        """

        # Query dst_graph
        dst_txn = self.dst_client.txn(read_only=True, best_effort=False)  # type: Txn
        try:
            res = dst_txn.query(query, variables, timeout, metadata, credentials)
            _res = json.loads(res.json)
            print(_res)
            # If any query has values, return res
            for response in _res.values():
                if response:
                    return res

            # Otherwise, try to copy from src to dst
            # Query source
            res = (
                self.src_client.txn(read_only=True).query(query, variables, timeout, metadata, credentials)
            )

            # If it isn't in the source, return the empty response
            _res = json.loads(res.json)
            if not any(_res.values()):
                return res

            # Otherwise, mutate the dst graph with the response
            mut_from_response(_res)
            dst_txn = self.dst_client.txn(read_only=False, best_effort=False)  # type: Txn

            dst_txn.mutate(set_obj=_res, commit_now=True)
        except Exception:
            dst_txn.discard()
            raise

        # Query dst_graph again
        return (
            self.dst_client.txn(read_only=True, best_effort=False)
                .query(query, variables, timeout, metadata, credentials)
        )


class CopyingDgraphClient(DgraphClient):
    def __init__(self, src_client: DgraphClient, dst_client: DgraphClient) -> None:
        super().__init__(*src_client._clients, *dst_client._clients)
        self.src_client = src_client
        self.dst_client = dst_client

    def txn(self, read_only=False, best_effort=False) -> CopyingTransaction:
        return CopyingTransaction(self)


class LensQuery(Queryable):
    def get_unique_predicate(self) -> Optional[str]:
        pass

    def get_node_type_name(self) -> Optional[str]:
        pass

    def get_node_key_filter(self) -> PropertyFilter:
        pass

    def get_uid_filter(self) -> PropertyFilter:
        pass

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        pass

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        pass

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        pass

    def __init__(self):
        super(LensQuery, self).__init__(LensView)


class LensView(Viewable):
    @staticmethod
    def get_property_tuples() -> List[Tuple[str, Callable[[Any], Union[str, int]]]]:
        pass

    @staticmethod
    def get_edge_tuples() -> List[Tuple[str, Union[List[Type[V]], Type[V]]]]:
        pass

    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: str,
            scope: List[NodeView],
            **kwargs: Dict[str, Any]
    ) -> None:
        super(LensView, self).__init__(dgraph_client, node_key, uid)
