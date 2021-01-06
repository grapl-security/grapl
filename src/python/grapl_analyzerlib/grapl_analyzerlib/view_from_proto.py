import json
from grapl_analyzerlib.prelude import (
    AssetView,
    BaseView,
    ProcessView,
    FileView,
    IpConnectionView,
    NetworkConnectionView,
    IpPortView,
    IpAddressView,
    ProcessOutboundConnectionView,
    ProcessInboundConnectionView,
)
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.retry import retry


def view_from_proto(graph_client: GraphClient, node) -> BaseView:
    if node.HasField("process_node"):
        uid = get_uid(graph_client, node.process_node.node_key)
        assert uid

        return ProcessView(
            graph_client=graph_client,
            uid=uid,
            node_key=node.process_node.node_key,
            node_types={"Process"},
        )
    elif node.HasField("file_node"):
        uid = get_uid(graph_client, node.file_node.node_key)

        return FileView(
            graph_client=graph_client,
            uid=uid,
            node_key=node.file_node.node_key,
            node_types={"File"},
        )
    elif node.HasField("asset_node"):
        uid = get_uid(graph_client, node.asset_node.node_key)

        return AssetView(
            uid, node.asset_node.node_key, graph_client, node_types={"Asset"}
        )
    elif node.HasField("ip_address_node"):
        uid = get_uid(graph_client, node.ip_address_node.node_key)

        return IpAddressView(
            uid, node.ip_address_node.node_key, graph_client, node_types={"IpAddress"},
        )
    elif node.HasField("ip_port_node"):
        uid = get_uid(graph_client, node.ip_port_node.node_key)

        return IpPortView(
            uid, node.ip_port_node.node_key, graph_client, node_types={"IpPort"}
        )
    elif node.HasField("process_outbound_connection_node"):
        uid = get_uid(graph_client, node.process_outbound_connection_node.node_key)
        return ProcessOutboundConnectionView(
            uid,
            node.process_outbound_connection_node.node_key,
            graph_client,
            node_types={"ProcessOutboundConnection"},
        )
    elif node.HasField("process_inbound_connection_node"):
        uid = get_uid(graph_client, node.process_inbound_connection_node.node_key)
        return ProcessInboundConnectionView(
            uid,
            node.process_inbound_connection_node.node_key,
            graph_client,
            node_types={"ProcessInboundConnection"},
        )
    elif node.HasField("ip_connection_node"):
        uid = get_uid(graph_client, node.ip_connection_node.node_key)
        return IpConnectionView(
            uid,
            node.ip_connection_node.node_key,
            graph_client,
            node_types={"IpConnection"},
        )
    elif node.HasField("network_connection_node"):
        uid = get_uid(graph_client, node.network_connection_node.node_key)
        return NetworkConnectionView(
            uid,
            node.network_connection_node.node_key,
            graph_client,
            node_types={"NetworkConnection"},
        )

    elif node.HasField("dynamic_node"):
        uid = get_uid(graph_client, node.dynamic_node.node_key)

        return BaseView(
            uid,
            node.dynamic_node.node_key,
            graph_client,
            node_types={node.dynamic_node.node_type},
        )
    else:
        raise Exception(f"Invalid Node Type : {node}")


# Proto nodes don't contain a uid so we have to fetch them. It may make sense to store these uids
# alongside the proto in the future. This makes constructing from proto relatively expensive.
@retry()
def get_uid(client: GraphClient, node_key: str) -> str:
    with client.txn_context(read_only=True) as txn:
        query = """
            query res($a: string)
            {
              res(func: eq(node_key, $a), first: 1) @cascade
               {
                 uid,
               }
             }"""
        res = txn.query(query, variables={"$a": node_key})
        res = json.loads(res.json)

        if isinstance(res["res"], list):
            if res["res"]:
                return str(res["res"][0]["uid"])
            else:
                raise Exception(f"get_uid failed for node_key: {node_key} {res}")
        else:
            return str(res["res"]["uid"])
