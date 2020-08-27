import time
import json
import pydgraph

from grpc import RpcError, StatusCode
from pydgraph import DgraphClient, DgraphClientStub
from grapl_analyzerlib.schemas import *

from grapl_analyzerlib.schemas.schema_builder import ManyToMany


def set_schema(client, schema, engagement=False):
    op = pydgraph.Operation(schema=schema)
    print(client.alter(op))


def drop_all(client):
    op = pydgraph.Operation(drop_all=True)
    client.alter(op)


def format_schemas(schema_defs):
    schemas = "\n\n".join([schema.to_schema_str() for schema in schema_defs])

    types = "\n\n".join([schema.generate_type() for schema in schema_defs])

    return "\n".join(
        [
            "  # Type Definitions",
            types,
            "\n  # Schema Definitions",
            schemas,
        ]
    )


def get_type_dict(client, type_name):
    query = f"""
    schema(type: {type_name}) {{
      type
      index
    }}
    """

    txn = client.txn(read_only=True)

    try:
        res = json.loads(txn.query(query).json)
    finally:
        txn.discard()

    type_dict = {}

    for d in res["types"][0]["fields"]:
        if d["name"][0] == "~":
            name = f"<{d['name']}>"
        else:
            name = d["name"]
        type_dict[name] = d["type"]

    return type_dict


def update_reverse_edges(client, schema):

    type_dicts = {}

    rev_edges = set()
    for edge in schema.forward_edges:
        edge_n = edge[0]
        edge_t = edge[1]._inner_type.self_type()
        if edge_t == "Any":
            continue

        rev_edges.add(("<~" + edge_n + ">", edge_t))

        if not type_dicts.get(edge_t):
            type_dicts[edge_t] = get_type_dict(client, edge_t)

    if not rev_edges:
        return

    for (rev_edge_n, rev_edge_t) in rev_edges:
        type_dicts[rev_edge_t][rev_edge_n] = "uid"

    type_strs = ""

    for t in type_dicts.items():
        type_name = t[0]
        type_d = t[1]

        predicates = []
        for predicate_name, predicate_type in type_d.items():
            predicates.append(f"\t{predicate_name}: {predicate_type}")

        predicates = "\n".join(predicates)
        type_str = f"""
type {type_name} {{
{predicates}
            
    }}
        """
        type_strs += "\n"
        type_strs += type_str

    op = pydgraph.Operation(schema=type_strs)
    client.alter(op)


def provision(mclient):

    # drop_all(mclient)
    # drop_all(___local_dg_provision_client)

    schemas = (
        AssetSchema(),
        ProcessSchema(),
        FileSchema(),
        IpConnectionSchema(),
        IpAddressSchema(),
        IpPortSchema(),
        NetworkConnectionSchema(),
        ProcessInboundConnectionSchema(),
        ProcessOutboundConnectionSchema(),
    )

    mg_schema_str = format_schemas(schemas)
    set_schema(mclient, mg_schema_str)


if __name__ == "__main__":

    local_dg_provision_client = DgraphClient(DgraphClientStub("localhost:9080"))

    num_retries = 150
    for i in range(0, num_retries):
        try:
            provision(
                local_dg_provision_client,
            )
        except Exception as e:
            hint = ""
            if isinstance(e, RpcError) and e.code() == StatusCode.UNAVAILABLE:
                hint = "have you booted the local dgraph server?"

            print(f"Retry {i}/{num_retries}: {hint}\n {e}")
            time.sleep(1)
        else:
            break

    time.sleep(1)
    print("grapl_provision complete!\n")
