import datetime
import logging
from typing import Dict, Iterable, Iterator, Optional

from chalice import Chalice

from grapl_analyzerlib.grapl_client import (
    GraphClient,
    LocalMasterGraphClient,
    MasterGraphClient,
)

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))
GRAPL_DGRAPH_TTL_S = int(os.environ.get("GRAPL_DGRAPH_TTL_S", "-1"))
GRAPL_LOG_LEVEL = os.environ.get("GRAPL_LOG_LEVEL", "ERROR")
GRAPL_TTL_DELETE_BATCH_SIZE = int(os.environ.get("GRAPL_TTL_DELETE_BATCH_SIZE", "100"))

app = Chalice(app_name="grapl-dgraph-ttl")
app.log.setLevel(GRAPL_LOG_LEVEL)


def query_batch(
    client: GraphClient,
    batch_size: int,
    ttl_cutoff_s: int,
    last_uid: Optional[str] = None,
) -> Dict[str, Any]:
    after = "" if last_uid is None else f" after: {last_uid}"
    paging = f"first: {batch_size}{after}"
    query = f"""
    {{
        q(func: le(last_index_time, {ttl_cutoff_s}) {paging}) {{
            uid,
            expand(_all_) {{ expand(_all_) }}
        }}
    }}
    """

    txn = client.txn()
    try:
        return txn.query(query)
    finally:
        txn.discard()


def calculate_ttl_cutoff_s(now: datetime.datetime, ttl_s: int) -> int:
    delta = datetime.timedelta(seconds=ttl_s)
    cutoff = now - delta
    return int(cutoff.timestamp())


def expired_node_uids(
    client: GraphClient, now: datetime.datetime, ttl_s: int, batch_size: int
) -> Iterator[Iterable[str]]:
    ttl_cutoff_s = calculate_ttl_cutoff_s(now, ttl_s)

    last_uid = None
    while 1:
        results = query_batch(client, batch_size, ttl_cutoff_s, last_uid)
        batch = []  # FIXME

        yield batch

        if len(batch) < batch_size:  # this is the last page
            break

        last_uid = None  # FIXME


def delete_batch(client: GraphClient, uids: Iterable[str]) -> None:
    del_ = {"delete": [{"uid": uid} for uid in uids]}

    txn = client.txn()
    try:
        mut = txn.create_mutation(del_obj=del_)
        txn.mutate(mutation=mut)
        txn.commit()
    finally:
        txn.discard()


@app.lambda_function(name="prune_expired_subgraphs")
def prune_expired_subgraphs() -> None:
    if GRAPL_DGRAPH_TTL_S > 0:
        for uids in expired_node_uids(
            client=LocalMasterGraphClient() if IS_LOCAL else MasterGraphClient(),
            now=datetime.datetime.utcnow(),
            ttl_s=GRAPL_DGRAPH_TTL_SECONDS,
            batch_size=GRAPL_TTL_DELETE_BATCH_SIZE,
        ):
            delete_batch(client, uids)
