import datetime
import json
import os

from typing import Dict, Iterable, Iterator, Optional, Tuple, Union

from chalice import Chalice

from grapl_analyzerlib.grapl_client import (
    GraphClient,
    LocalMasterGraphClient,
    MasterGraphClient,
)

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))
GRAPL_DGRAPH_TTL_S = int(os.environ.get("GRAPL_DGRAPH_TTL_S", "-1"))
GRAPL_LOG_LEVEL = os.environ.get("GRAPL_LOG_LEVEL", "ERROR")
GRAPL_TTL_DELETE_BATCH_SIZE = int(os.environ.get("GRAPL_TTL_DELETE_BATCH_SIZE", "1000"))

app = Chalice(app_name="grapl-dgraph-ttl")
app.log.setLevel(GRAPL_LOG_LEVEL)


def query_batch(
    client: GraphClient,
    batch_size: int,
    ttl_cutoff_ms: int,
    last_uid: Optional[str] = None,
) -> Iterable[Dict[str, Union[Dict, str]]]:
    after = "" if last_uid is None else f", after: {last_uid}"
    paging = f"first: {batch_size}{after}"
    query = f"""
    {{
        q(func: le(last_index_time, {ttl_cutoff_ms}), {paging}) {{
            uid,
            expand(_all_) {{ uid }}
        }}
    }}
    """

    txn = client.txn()
    try:
        app.log.debug(f"retrieving batch: {query}")
        batch = txn.query(query)
        app.log.debug(f"retrieved batch: {batch.json}")
        return json.loads(batch.json)["q"]
    finally:
        txn.discard()


def calculate_ttl_cutoff_ms(now: datetime.datetime, ttl_s: int) -> int:
    delta = datetime.timedelta(seconds=ttl_s)
    cutoff = now - delta
    return int(cutoff.timestamp() * 1000)


def expired_entities(
    client: GraphClient, now: datetime.datetime, ttl_s: int, batch_size: int
) -> Iterator[Iterable[Dict[str, Union[Dict, str]]]]:
    ttl_cutoff_ms = calculate_ttl_cutoff_ms(now, ttl_s)

    app.log.info(f"Pruning entities last indexed before {ttl_cutoff_ms}")

    last_uid = None
    while 1:
        results = query_batch(client, batch_size, ttl_cutoff_ms, last_uid)

        if len(results) > 0:
            last_uid = results[-1]["uid"]
            yield results

        if len(results) < batch_size:
            break  # this was the last page of results


def nodes(entities: Iterable[Dict[str, Union[Dict, str]]]) -> Iterator[str]:
    for entity in entities:
        yield entity["uid"]


def edges(
    entities: Iterable[Dict[str, Union[Dict, str]]]
) -> Iterator[Tuple[str, str, str]]:
    for entity in entities:
        uid = entity["uid"]
        for key, value in entity.items():
            if isinstance(value, list):
                for v in value:
                    if isinstance(v, dict):
                        if len(v.keys()) == 1 and "uid" in v.keys():
                            yield (uid, key, v["uid"])


def delete_nodes(client: GraphClient, nodes: Iterator[str]) -> int:
    del_ = [{"uid": uid} for uid in nodes]

    txn = client.txn()
    try:
        mut = txn.create_mutation(del_obj=del_)
        app.log.debug(f"deleting nodes: {mut}")
        txn.mutate(mutation=mut, commit_now=True)
        app.log.debug(f"deleted nodes: {json.dumps(del_)}")
        return len(del_)
    finally:
        txn.discard()


def delete_edges(client: GraphClient, edges: Iterator[Tuple[str, str, str]]) -> int:
    del_ = [
        create_edge_obj(src_uid, predicate, dest_uid)
        for src_uid, predicate, dest_uid in edges
    ]

    txn = client.txn()
    try:
        mut = txn.create_mutation(del_obj=del_)
        app.log.debug(f"deleting edges: {mut}")
        txn.mutate(mutation=mut, commit_now=True)
        app.log.debug(f"deleted edges: {json.dumps(del_)}")
        return len(del_)
    finally:
        txn.discard()


def create_edge_obj(
    src_uid: str, predicate: str, dest_uid: str
) -> Dict[str, Union[Dict, str]]:
    if predicate.startswith("~"):  # this is a reverse edge
        return {"uid": dest_uid, predicate.lstrip("~"): {"uid": src_uid}}
    else:  # this is a forward edge
        return {"uid": src_uid, predicate: {"uid": dest_uid}}


@app.lambda_function(name="prune_expired_subgraphs")
def prune_expired_subgraphs(event, lambda_context) -> None:
    if GRAPL_DGRAPH_TTL_S > 0:
        client = LocalMasterGraphClient() if IS_LOCAL else MasterGraphClient()

        node_count = 0
        edge_count = 0

        for entities in expired_entities(
            client,
            now=datetime.datetime.utcnow(),
            ttl_s=GRAPL_DGRAPH_TTL_S,
            batch_size=GRAPL_TTL_DELETE_BATCH_SIZE,
        ):
            edge_count += delete_edges(client, edges(entities))
            node_count += delete_nodes(client, nodes(entities))

        app.log.info(f"Pruned {node_count} nodes and {edge_count} edges")
    else:
        app.log.warn("GRAPL_DGRAPH_TTL_S is not set, exiting.")


if IS_LOCAL:
    import time

    while 1:
        time.sleep(60)
        prune_expired_subgraphs()
