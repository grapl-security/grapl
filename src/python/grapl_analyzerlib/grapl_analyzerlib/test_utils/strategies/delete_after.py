from typing import Iterator, TypeVar
from grapl_analyzerlib.grapl_client import GraphClient


def delete(graph_client: GraphClient, uid: str) -> None:
    mut = f"""
        {{
            delete {{
                {uid} * *
            }}
        }}
    """
    with graph_client.txn_context(read_only=False) as txn:
        txn.mutate(mut)
