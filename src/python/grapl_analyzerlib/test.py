import json
from copy import deepcopy
from typing import *

from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzerlib.nodes.comparators import IntCmp, _str_cmps, StrCmp, _int_cmps
from grapl_analyzerlib.nodes.dynamic_node import DynamicNodeQuery, DynamicNodeView
from grapl_analyzerlib.nodes.external_ip_node import ExternalIpView
from grapl_analyzerlib.nodes.process_node import ProcessQuery, ProcessView, IProcessView
from grapl_analyzerlib.nodes.types import Property, PropertyT
from grapl_analyzerlib.nodes.viewable import Viewable, EdgeViewT, ForwardEdgeView

T = TypeVar('T')


def create_edge(client: DgraphClient, from_uid: str, edge_name: str, to_uid: str) -> None:
    if edge_name[0] == '~':
        mut = {
            'uid': to_uid,
            edge_name[1:]: {'uid': from_uid}
        }

    else:
        mut = {
            'uid': from_uid,
            edge_name: {'uid': to_uid}
        }

    txn = client.txn(read_only=False)
    try:
        txn.mutate(set_obj=mut, commit_now=True)
    finally:
        txn.discard()


def _upsert(client: DgraphClient, node_dict: Dict[str, Property]) -> str:
    if node_dict.get('uid'):
        node_dict.pop('uid')
    node_dict['uid'] = '_:blank-0'
    node_key = node_dict['node_key']
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}")) {{
                    uid,  
                    expand(_forward_)            
            }}
        }}
        """
    txn = client.txn(read_only=False)

    try:
        res = json.loads(txn.query(query).json)['q0']
        new_uid = None
        if res:
            node_dict['uid'] = res[0]['uid']
            new_uid = res[0]['uid']

        mutation = node_dict

        m_res = txn.mutate(set_obj=mutation, commit_now=True)
        uids = m_res.uids

        if new_uid is None:
            new_uid = uids['blank-0']
        return str(new_uid)

    finally:
            txn.discard()


def upsert(
        client: DgraphClient,
        view_type: Type[Viewable],
        node_key: str,
        node_props: Dict[str, Property]
) -> Viewable:
    node_props['node_key'] = node_key
    uid = _upsert(client, node_props)
    # print(f'uid: {uid}')
    node_props['uid'] = uid
    # print(node_props['node_key'])
    return view_type.from_dict(client, node_props)


class IpcQuery(DynamicNodeQuery):
    def __init__(self) -> None:
        super(IpcQuery, self).__init__("Ipc", IpcView)

    def with_src_pid(self, eq=IntCmp, gt=IntCmp, lt=IntCmp) -> "IpcQuery":
        self.set_int_property_filter("src_pid", _int_cmps("src_pid", eq=eq, gt=gt, lt=lt))
        return self

    def with_dst_pid(self, eq=IntCmp, gt=IntCmp, lt=IntCmp) -> "IpcQuery":
        self.set_int_property_filter("dst_pid", _int_cmps("dst_pid", eq=eq, gt=gt, lt=lt))
        return self

    def with_ipc_type(self, eq=StrCmp, contains=StrCmp, ends_with=StrCmp) -> "IpcQuery":
        self.set_str_property_filter(
            "ipc_type", _str_cmps("ipc_type", eq=eq, contains=contains, ends_with=ends_with)
        )
        return self

    def with_ipc_creator(
            self, ipc_creator_query: "Optional[ProcessQuery]" = None
    ) -> "IpcQuery":
        if ipc_creator_query:
            ipc_creator = deepcopy(ipc_creator_query)
        else:
            ipc_creator = ProcessQuery()
        self.set_forward_edge_filter("ipc_creator", ipc_creator)
        ipc_creator.set_reverse_edge_filter("~ipc_creator", self, "ipc_creator")
        return self

    def with_ipc_recipient(
            self, ipc_recipient_query: "Optional[ProcessQuery]" = None
    ) -> "IpcQuery":
        if ipc_recipient_query:
            ipc_recipient = deepcopy(ipc_recipient_query)
        else:
            ipc_recipient = ProcessQuery()

        self.set_forward_edge_filter("ipc_recipient", ipc_recipient)
        ipc_recipient.set_reverse_edge_filter("~ipc_recipient", self, "ipc_recipient")
        return self


class IpcView(DynamicNodeView):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: str,
            node_type: str,
            src_pid: Optional[int] = None,
            dst_pid: Optional[int] = None,
            ipc_creator: Optional[IProcessView] = None,
            ipc_recipient: Optional[IProcessView] = None,
            **kwargs
    ) -> None:
        super(IpcView, self).__init__(
            dgraph_client=dgraph_client, node_key=node_key, uid=uid, node_type=node_type
        )

        self.node_type = node_type
        self.src_pid = src_pid
        self.dst_pid = dst_pid
        self.ipc_creator = ipc_creator
        self.ipc_recipient = ipc_recipient

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {"src_pid": int, "dst_pid": int}

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        f_edges = {"ipc_creator": ProcessView, "ipc_recipient": ProcessView}

        return {fe[0]: fe[1] for fe in f_edges.items() if fe[1]}

    def _get_forward_edges(self) -> "Mapping[str, ForwardEdgeView]":
        f_edges = {"ipc_creator": self.ipc_creator, "ipc_recipient": self.ipc_recipient}

        return {fe[0]: fe[1] for fe in f_edges.items() if fe[1]}

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        props = {"src_pid": self.src_pid, "dst_pid": self.dst_pid}
        return {p[0]: p[1] for p in props.items() if p[1] is not None}


def test_ipc(local_client: DgraphClient):
    ipc = {
        "dgraph.type": "Ipc",
        "ipc_type": "UNIX_DOMAIN",
    }  # type: Dict[str, Property]

    ipc_view = upsert(
        local_client,
        IpcView,
        '6fadeb67-4b20-4870-b848-647e97bc5543',
        ipc
    )

    qv = IpcQuery().with_ipc_type(eq="UNIX_DOMAIN")
    # print(
    #     generate_query(
    #         query_name='qname',
    #         root=qv,
    #         binding_modifier='bm',
    #     )
    # )
    # print(qv.query_first(local_client))
    # # print(qv)
    # print(ipc)
    # print(ipc_view)

def main() -> None:
    local_client = DgraphClient(DgraphClientStub('localhost:9080'))

    test_ipc(local_client)

    parent = {
        'process_id': 100,
        'process_name': 'word.exe'
    }  # type: Dict[str, Property]

    child = {
        'process_id': 1234,
        'process_name': 'cmd.exe'
    }  # type: Dict[str, Property]

    external_ip = {
        'external_ip': '56.123.14.24',
    }  # type: Dict[str, Property]



    parent_view = upsert(
        local_client,
        ProcessView,
        'ea75f056-61a1-479d-9ca2-f632d2c67205',
        parent
    )

    child_view = upsert(
        local_client,
        ProcessView,
        '10f585c2-cf31-41e2-8ca5-d477e78be3ac',
        child
    )

    external_ip_view = upsert(
        local_client,
        ExternalIpView,
        '8bc20354-e8c5-49fc-a984-2927b24c1a29',
        external_ip
    )


    create_edge(local_client, parent_view.uid, 'children', child_view.uid)
    create_edge(local_client, child_view.uid, 'created_connections', external_ip_view.uid)


    queried_child_0 = ProcessQuery().with_process_id(eq=1234).query_first(local_client)

    assert queried_child_0
    assert queried_child_0.node_key == child_view.node_key

    queried_child_1 = (
        ProcessQuery()
            .with_process_id(eq=1234)
            .query_first(local_client, contains_node_key='10f585c2-cf31-41e2-8ca5-d477e78be3ac')
    )

    assert queried_child_1
    assert queried_child_1.node_key == child_view.node_key
    assert queried_child_1.node_key == queried_child_0.node_key

    p = (
        ProcessQuery()
        .with_process_name(eq="cmd.exe")
        .with_parent()
        .with_created_connection()
        .query_first(local_client)
     )  # type: Optional[ProcessView]

    assert p



if __name__ == '__main__':
    main()
