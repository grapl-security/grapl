from __future__ import annotations

import json
from typing import Iterable, Callable, List, Dict, Any, Union, Tuple

import itertools
from pydgraph import DgraphClient

from graph import batch_queries

import graph_description_pb2

class Subgraph(object):
    def __init__(self, s: bytes):
        self.subgraph = graph_description_pb2.GraphDescription()
        self.subgraph.ParseFromString(s)


class NodeRef(object):
    def __init__(self, uid, node_type):
        self.uid = uid
        self.node_type = node_type

    def to_dict(self):
        # type: () -> Dict[str, Any]
        return {
            'uid': self.uid,
            'node_type': self.node_type
        }


class ExecutionHit(object):
    def __init__(self, label: str, node_refs: List[NodeRef], edges: List[Tuple[str, str, str]]) -> None:
        self.label = label
        self.node_refs = node_refs
        self.edges = edges

    def to_json(self):
        # type: () -> str
        return json.dumps(
            {
                'label': self.label,
                'node_refs': [n.to_dict() for n in self.node_refs],
                'edges': self.edges
            }
        )

    @staticmethod
    def from_parent_child(label: str, hit: Dict[str, Any]) -> Any:
        child_uid = NodeRef(hit['children'][0]['uid'], 'Process')
        parent_uid = NodeRef(hit['uid'], 'Process')

        return ExecutionHit(
            label,
            [parent_uid, child_uid],
            [(parent_uid.uid, "children", child_uid.uid)]
        )


class ExecutionMiss(object):
    pass


class ExecutionComplete(object):
    pass


ExecutionResult = Union[ExecutionHit, ExecutionMiss, ExecutionComplete]


class ExecutionEvent(object):
    def __init__(self, key: str, subgraph: Subgraph) -> None:
        self.key = key
        self.subgraph = subgraph


def analyze_by_node_key(client: DgraphClient,
                        keys: Union[Iterable[str], Subgraph],
                        signature_fn: Callable[[str], str]) -> List[Dict[str, Any]]:
    if isinstance(keys, Subgraph):
        keys = [node_key for node_key in keys.subgraph.nodes]
    queries = [signature_fn(node_key) for node_key in keys]
    batched = batch_queries(queries)
    response = json.loads(client.query(batched).json)
    return list(itertools.chain.from_iterable(response.values()))
