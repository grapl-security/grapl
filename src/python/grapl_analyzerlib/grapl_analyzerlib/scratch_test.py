import json

from typing import Any, Dict, List, Optional, Union, Type

import pydgraph
from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzerlib.viewable import Viewable
from grapl_analyzerlib.nodes.process import (
    ProcessQuery,
    ProcessView,
)
from grapl_analyzerlib.nodes.ip_port import (
    IpPortQuery,
    IpPortView,
)
from grapl_analyzerlib.nodes.ip_address import (
    IpAddressQuery,
    IpAddressView,
)

from grapl_analyzerlib.nodes.process_user import (
    ProcessWithUserQuery,
    ProcessWithUserView,
)

from grapl_analyzerlib.nodes.file import (
    FileQuery,
    FileView,
    FileExtendsProcessQuery,
    FileExtendsProcessView,
)

from grapl_analyzerlib.nodes.lens import LensView, LensQuery, LensSchema

ProcessQuery = ProcessQuery.extend_self(FileExtendsProcessQuery)
ProcessView = ProcessView.extend_self(FileExtendsProcessView)


def set_schema(client, schema, engagement=False):
    op = pydgraph.Operation(schema=schema)
    print(client.alter(op))


def create_edge(
    client: DgraphClient, from_uid: str, edge_name: str, to_uid: str
) -> None:
    if edge_name[0] == "~":
        mut = {"uid": to_uid, edge_name[1:]: {"uid": from_uid}}

    else:
        mut = {"uid": from_uid, edge_name: {"uid": to_uid}}

    txn = client.txn(read_only=False)
    try:
        txn.mutate(set_obj=mut, commit_now=True)
    finally:
        txn.discard()


def _upsert(client: DgraphClient, node_dict: Dict[str, Any]) -> str:
    if node_dict.get("uid"):
        node_dict.pop("uid")
    node_dict["uid"] = "_:blank-0"
    node_key = node_dict["node_key"]
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}")) {{
                    uid,
                    dgraph.type
                    expand(_all_)
            }}
        }}
        """
    txn = client.txn(read_only=False)

    try:
        res = json.loads(txn.query(query).json)["q0"]
        new_uid = None
        if res:
            node_dict["uid"] = res[0]["uid"]
            new_uid = res[0]["uid"]

        mutation = node_dict

        m_res = txn.mutate(set_obj=mutation, commit_now=True)
        uids = m_res.uids

        if new_uid is None:
            new_uid = uids["blank-0"]
        return str(new_uid)

    finally:
        txn.discard()


def upsert(
    client: DgraphClient,
    type_name: str,
    view_type: Type[Viewable],
    node_key: str,
    node_props: Dict[str, Any],
) -> Viewable:
    node_props["node_key"] = node_key
    node_props["dgraph.type"] = [type_name]
    uid = _upsert(client, node_props)
    # print(f'uid: {uid}')
    node_props["uid"] = uid
    # print(node_props['node_key'])
    return view_type.from_dict(node_props, client)


def main():
    local_client = DgraphClient(DgraphClientStub("localhost:9080"))

    lens = {"lens": "shared_hostname"}

    grand_parent = {"process_name": "explorer.exe", "user": "okuser"}
    parent = {"process_name": "word.exe", "user": "okuser"}

    parent_bin = {
        "file_path": "word.exe",
    }
    child = {"process_name": "cmd.exe"}  # type: Dict[str, Property]

    child2 = {
        "process_name": "evil.exe",
        "user": "eviluser",
    }  # type: Dict[str, Property]

    parent_bin_view = upsert(
        local_client,
        "File",
        FileView,
        "cec290c8-6dd2-4ec4-a5f3-6584d36c963b",
        parent_bin,
    )

    grand_parent_view = upsert(
        local_client,
        "Process",
        ProcessView,
        "e03519f6-6f84-4f87-b426-196e249e7b7a",
        grand_parent,
    )

    parent_view = upsert(
        local_client,
        "Process",
        ProcessView,
        "ea75f056-61a1-479d-9ca2-f632d2c67205",
        parent,
    )

    child_view = upsert(
        local_client,
        "Process",
        ProcessView,
        "10f585c2-cf31-41e2-8ca5-d477e78be3ac",
        child,
    )

    child_view2 = upsert(
        local_client,
        "Process",
        ProcessView,
        "251502ab-3332-4225-a0ec-128ea17c51d2",
        child2,
    )

    lens_view = upsert(
        local_client, "Lens", LensView, "0b1da9a3-b16c-4d6b-8b45-18e474a58ed0", lens,
    )

    create_edge(local_client, parent_view.uid, "bin_file", parent_bin_view.uid)
    create_edge(local_client, parent_bin_view.uid, "spawned_from", parent_view.uid)

    create_edge(local_client, grand_parent_view.uid, "children", parent_view.uid)
    create_edge(local_client, parent_view.uid, "parent", grand_parent_view.uid)

    create_edge(local_client, parent_view.uid, "children", child_view.uid)
    create_edge(local_client, parent_view.uid, "children", child_view2.uid)

    create_edge(local_client, child_view.uid, "parent", parent_view.uid)
    create_edge(local_client, child_view2.uid, "parent", parent_view.uid)

    create_edge(local_client, lens_view.uid, "scope", parent_view.uid)
    create_edge(local_client, lens_view.uid, "scope", parent_bin_view.uid)
    create_edge(local_client, lens_view.uid, "scope", child_view.uid)
    create_edge(local_client, lens_view.uid, "scope", child_view2.uid)
    create_edge(local_client, parent_view.uid, "in_scope", lens_view.uid)
    create_edge(local_client, parent_bin_view.uid, "in_scope", lens_view.uid)
    create_edge(local_client, child_view.uid, "in_scope", lens_view.uid)
    create_edge(local_client, child_view2.uid, "in_scope", lens_view.uid)

    p = (
        ProcessQuery()
        .with_lenses()
        .with_parent()
        .with_user()
        .with_bin_file()
        .with_process_name()
        .with_children(
            ProcessQuery().with_process_name(eq="cmd.exe"),
            ProcessQuery().with_process_name(eq="evil.exe"),
        )
    )
    print("---")
    print(ProcessQuery.__dict__)
    for node_key in (
        "10f585c2-cf31-41e2-8ca5-d477e78be3ac",
        "ea75f056-61a1-479d-9ca2-f632d2c67205",
        "251502ab-3332-4225-a0ec-128ea17c51d2",
        "e03519f6-6f84-4f87-b426-196e249e7b7a",
    ):
        pv = p.query_first(local_client, contains_node_key=node_key)

        # print(pv.node_key)
        print("get_user", pv.get_user())
        print("get_bin_file", pv.get_bin_file())
        print("bin_spawned", pv.get_bin_file().get_spawned_from())
        print(pv.parent.get_process_name())
        print(pv.children[0].process_name)
        print(pv.children[1].process_name)

    # break

    print("-----")
    l = LensQuery().with_scope().query_first(local_client)
    print(l)
    if not l:
        raise Exception('Expected lens')
    for scoped in l.scope:
        print(scoped)
        maybe_proc = scoped.into_view(ProcessView)
        if maybe_proc:
            print("lens", maybe_proc.get_lenses())


if __name__ == "__main__":
    gclient = DgraphClient(DgraphClientStub("localhost:9080"))

    set_schema(
        gclient,
        """
        type Process {
            process_name
            node_key
            children
            parent
            user
            
            # Attached by the File node
            bin_file
            created_files
            wrote_files
            read_files
            deleted_files
            
            in_scope
        }
        
        type File {
            node_key
            file_path
            spawned_from
            creator
            writers
            readers
            deleter
            in_scope
        }
                
        type Lens {
            node_key
            lens
            scope
        }
        
        process_name: string @index(exact, trigram) .
        file_path: string @index(exact, trigram) .
        user: string @index(exact, trigram) .
        node_key: string @index(hash) @upsert .
        children: [uid] .
        parent: uid .
        
        lens: string @index(exact) .
        
        spawned_from: [uid] .
        bin_file: uid .
        
        created_files: [uid] .
        wrote_files: [uid] .
        read_files: [uid] .
        deleted_files: [uid] .
        
        creator: uid .
        writers: [uid] .
        readers: [uid] .
        deleter: uid .
        
        scope: [uid] .
        in_scope: [uid] .
    """,
    )

    main()
