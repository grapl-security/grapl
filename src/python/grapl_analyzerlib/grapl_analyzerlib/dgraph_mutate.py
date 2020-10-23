import json
from typing import (
    Any,
    Dict,
    cast,
    TypeVar,
    Type,
)

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.viewable import Viewable
from grapl_analyzerlib.queryable import Queryable

V = TypeVar("V", bound=Viewable)
Q = TypeVar("Q", bound=Queryable)


def _upsert(client: GraphClient, node_dict: Dict[str, Any]) -> str:
    node_dict["uid"] = "_:blank-0"
    node_key = node_dict["node_key"]
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}"), first: 1) {{
                    uid,
                    dgraph.type
                    expand(_all_)
            }}
        }}
        """

    with client.txn_context(read_only=False) as txn:
        res = json.loads(txn.query(query).json)["q0"]
        new_uid = None
        if res:
            node_dict["uid"] = res[0]["uid"]
            new_uid = res[0]["uid"]

        mutation = node_dict

        mut_res = txn.mutate(set_obj=mutation, commit_now=True)
        new_uid = node_dict.get("uid") or mut_res.uids["blank-0"]
        return cast(str, new_uid)


def upsert(
    client: GraphClient,
    type_name: str,
    view_type: Type[Viewable[V, Q]],
    node_key: str,
    node_props: Dict[str, Any],
) -> Viewable[V, Q]:
    node_props["node_key"] = node_key
    node_props["dgraph.type"] = list({type_name, "Base", "Entity"})
    uid = _upsert(client, node_props)
    node_props["uid"] = uid
    return view_type.from_dict(node_props, client)
