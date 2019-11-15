import time
import json
import pydgraph
from pydgraph import DgraphClient, DgraphClientStub


def set_process_schema(client: DgraphClient, engagement: bool = False):
    
    schema = """node_key: string @upsert @index(hash) .
    process_id: int @index(int) .
    created_timestamp: int @index(int) .
    asset_id: string @index(hash) .
    terminate_time: int @index(int) .
    image_name: string @index(exact, hash, trigram, fulltext) .
    process_name: string @index(exact, hash, trigram, fulltext) .
    arguments: string  @index(fulltext)  @index(trigram) .
    bin_file: uid @reverse .
    children: [uid] @reverse .
    created_files: [uid] @reverse .
    deleted_files: [uid] @reverse .
    read_files: [uid] @reverse .
    wrote_files: [uid] @reverse .
    created_connections: [uid] @reverse .
    bound_connections: [uid] @reverse .
    """
    
    if engagement:
        schema += "\n"
        schema += "risks: [uid] @reverse ."
        
    # unstable
    schema += """
        process_guid: string @index(exact, hash, trigram, fulltext) .
    """    
    
    op = pydgraph.Operation(schema=schema)
    client.alter(op)


def set_file_schema(client: DgraphClient, engagement: bool=False) -> None:
    
    schema = """
    node_key: string @upsert @index(hash) .
    file_name: string @index(exact, hash, trigram, fulltext) .
    asset_id: string @index(exact, hash, trigram, fulltext) .
    file_path: string @index(exact, hash, trigram, fulltext) .
    file_extension: string @index(exact, hash, trigram, fulltext) .
    file_mime_type: string @index(exact, hash, trigram, fulltext) .
    file_size: int @index(int) .
    file_version: string @index(exact, hash, trigram, fulltext) .
    file_description: string @index(exact, hash, trigram, fulltext) .
    file_product: string @index(exact, hash, trigram, fulltext) .
    file_company: string @index(exact, hash, trigram, fulltext) .
    file_directory: string @index(exact, hash, trigram, fulltext) .
    file_inode: int @index(int) .
    file_hard_links: string @index(exact, hash, trigram, fulltext) .
    signed: bool @index(bool) .
    signed_status: string @index(exact, hash, trigram, fulltext) .
    md5_hash: string @index(exact, hash, trigram, fulltext) .
    sha1_hash: string @index(exact, hash, trigram, fulltext) .
    sha256_hash: string @index(exact, hash, trigram, fulltext) .
    """
    if engagement:
        schema += "\n"
        schema += "risks: uid @reverse ."
        
    op = pydgraph.Operation(schema=schema)
    client.alter(op)
    
def set_outbound_connection_schema(client, engagement=False):
    schema = """
    
        create_time: int @index(int) .
        terminate_time: int @index(int) .
        last_seen_time: int @index(int) .
        ip: string @index(exact, trigram, hash) .
        port: string @index(exact, trigram, hash) .
    """
    if engagement:
        schema += "\n"
        schema += "risks: [uid] @reverse ."
        
    op = pydgraph.Operation(schema=schema)
    client.alter(op)
    
    
def set_inbound_connection_schema(client, engagement=False):
    schema = """
        node_key: string @upsert @index(hash) .
        asset_id: string @index(exact, hash, trigram, fulltext) .
        port: string @index(exact, trigram, hash) .
    """
    if engagement:
        schema += "\n"
        schema += "risks: [uid] @reverse ."
        
    op = pydgraph.Operation(schema=schema)
    client.alter(op)
    
    
def set_external_ip_schema(client, engagement=False):
    schema = """
        node_key: string @upsert @index(hash) .

        external_ip: string @index(exact, trigram, hash) .
    """
    if engagement:
        schema += "\n"
        schema += "risks: [uid] @reverse ."
        
    op = pydgraph.Operation(schema=schema)
    client.alter(op)
    
    
def set_risk_schema(client, engagement=False):
    schema = """
        analyzer_name: string @index(exact, trigram, hash) .
        risk_score: int @index(int) .
    """
    op = pydgraph.Operation(schema=schema)
    client.alter(op)
    
def set_lens_schema(client, engagement=False):
    schema = """
        scope: [uid] @reverse .
        lens: string @upsert @index(exact, trigram, hash) .
        score: int @index(int) .
    """
    op = pydgraph.Operation(schema=schema)
    client.alter(op)
    
def set_ipc_schema(client, engagement=False):
    schema = """

        type Ipc {
            node_key: string
            key: string
            ipc_type: string
            src_pid: int
            dst_pid: int
            ipc_creator: uid
            ipc_recipient: uid
        }

        node_key: string @upsert @index(hash) .

        ipc_type: string @index(hash) .
        src_pid: int @index(int) .
        dst_pid: int @index(int) .
        ipc_creator: uid @reverse .
        ipc_recipient: uid @reverse .
"""
    
    if engagement:
        schema += "\n"
        schema += "risks: [uid] @reverse ."
        
    op = pydgraph.Operation(schema=schema)
    client.alter(op)
    
    
def drop_all(client):
    op = pydgraph.Operation(drop_all=True)
    client.alter(op)

client = DgraphClient(DgraphClientStub('localhost:9080'))
drop_all(client)

set_process_schema(client)
set_file_schema(client)
set_outbound_connection_schema(client)
set_inbound_connection_schema(client)
set_external_ip_schema(client)
set_ipc_schema(client)