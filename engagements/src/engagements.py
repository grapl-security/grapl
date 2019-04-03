import json

from typing import Dict, Optional, List

from pydgraph import DgraphClient, DgraphClientStub

Process = int

class NodeCopier(object):

    def __init__(self, engagement_key: str, mg_client: DgraphClient, eg_client: DgraphClient):
        self.engagement_key = engagement_key
        self.mg_client = mg_client
        self.eg_client = eg_client

    @staticmethod
    def upsert(client: DgraphClient, node_key: str, props: Dict[str, str]):
        query = """
            query q0($a: string)
            {
              q0(func: eq(node_key, $a))
              {
                uid,
                node_key,
                image_name,
                image_path,
              }
            }
            """

        txn = client.txn(read_only=False)

        try:
            res = json.loads(txn.query(query, variables={'$a': node_key}).json)
            node = res['q0']

            if not node:
                node = props
            else:
                node = {**props, **node[0]}
            res = txn.mutate(set_obj=node, commit_now=True)
            uids = res.uids
            uid = uids['blank-0'] or node['uid']

        finally:
            txn.discard()

        return uid

    def copy_node(self, node_uid: str) -> str:
        query = """
            query q0($a: string)
            {
              q0(func: uid($a))
              {
                  uid,
                  node_key,
                  image_name,
                  image_path
              }
            }
            """

        res = json.loads(self.mg_client.query(query, variables={'$a': node_uid}).json)
        # We assume the node exists in the master graph
        node = res['q0'][0]

        # Prep node for upsert into engagement
        node.pop('uid', None)
        node['engagement_key'] = str(self.engagement_key)

        # Insert node into engagement-graph
        return NodeCopier.upsert(self.eg_client, node['node_key'], node)

    def copy_edge(self, from_uid: str, edge_name: str, to_uid: str):
        mut = {
            'uid': from_uid,
            edge_name: {'uid': to_uid}
        }

        res = self.eg_client.txn(read_only=False).mutate(set_obj=mut, commit_now=True)


class Engagement(object):
    def __init__(self, id: str):
        self.engagement_key = id
        self.eg_client : DgraphClient = DgraphClient(DgraphClientStub("db.engagementgraph:9080"))
        self.mg_client : DgraphClient = DgraphClient(DgraphClientStub("db.mastergraph:9080"))
        self.node_copier : NodeCopier = NodeCopier(id, self.mg_client, self.eg_client)

    def get_process_node(self, node_key) -> Process:
        proc_res = json.loads(self.eg_client.query(
            """{{
                q0(func: eq(node_key, "{}"))
                @filter(eq(engagement_key, "{}"))
                {{
                    uid,
                    node_key,
                    image_name,
                    image_path,
                }}
            }}""".format(node_key, self.engagement_key)
        ).json)['q0']

        return Process(
            proc_res[0]['node_key'],
            proc_res[0]['uid'],
            self.engagement_key,
            proc_res[0].get('image_name', None),
            proc_res[0].get('image_path', None),
            self,
        )


class Process(object):
    def __init__(
            self,
            node_key: str,
            uid: str,
            engagement_key: str,
            image_name: Optional[str],
            image_path: Optional[str],
            engagement: Engagement,
    ):
        self.node_key = node_key
        self.uid = uid
        self.engagement_key = engagement_key
        self.image_name = image_name
        self.image_path = image_path
        self.engagement = engagement

    def from_dict(self, d):
        return Process(
            d['node_key'],
            d['uid'],
            self.engagement_key,
            d.get('image_name', None),
            d.get('image_path', None),
            self.engagement,
        )

    def add_parent(self):
        mg_parent = self._get_parent(self.engagement.mg_client, False)

        eg_parent_uid = self.engagement.node_copier.copy_node(mg_parent.uid)
        self.engagement.node_copier.copy_edge(eg_parent_uid, 'children', self.uid)

        return self._get_parent(self.engagement.eg_client)

    def _get_parent(self, client: DgraphClient, eg=True) -> Optional[Process]:
        if eg:
            q_filter = '@filter(eq(engagement_key, "{}"))'.format(self.engagement_key)
        else:
            q_filter = ''

        proc_res = json.loads(client.query(
            """{{
                q0(func: eq(node_key, "{}"))
                {}
                {{
                    ~children {{
                        uid,
                        node_key,
                        image_name,
                        image_path,
                    }}
                    
                }}
            }}""".format(self.node_key, q_filter)
        ).json)['q0']

        if not proc_res:
            return None

        return Process(
            proc_res[0]['~children'][0]['node_key'],
            proc_res[0]['~children'][0]['uid'],
            self.engagement_key,
            proc_res[0]['~children'][0].get('image_name', None),
            proc_res[0]['~children'][0].get('image_path', None),
            self.engagement,
        )

    def get_parent(self) -> Optional[Process]:
        parent = self._get_parent(self.engagement.eg_client)

        return parent or self.add_parent()

    def add_children(self) -> Optional[List[Process]]:
        mg_children = self._get_children(self.engagement.mg_client, False)
        for mg_child in mg_children:
            eg_child_uid = self.engagement.node_copier.copy_node(mg_child.uid)
            self.engagement.node_copier.copy_edge(self.uid, 'children', eg_child_uid)

        return self._get_children(self.engagement.eg_client)

    def _get_children(self, client: DgraphClient, eg=True) -> Optional[List[Process]]:
        if eg:
            q_filter = '@filter(eq(engagement_key, "{}"))'.format(self.engagement_key)
        else:
            q_filter = ''

        proc_res = json.loads(client.query(
            """{{
                q0(func: eq(node_key, "{}"))
                {}
                {{
                    children {{
                        uid,
                        node_key,
                        image_name,
                        image_path,
                    }}
                    
                }}
            }}""".format(self.node_key, q_filter)
        ).json)['q0']

        if not proc_res:
            return None

        return [self.from_dict(child) for child in proc_res[0]['children']]

    def get_children(self):
        children = self._get_children(self.engagement.eg_client)
        return children or self.add_children()


def main():
    engagement = Engagement("0b30f2a75bb57f7c3282f52b44be6039609f2575")

    svchost = engagement.get_process_node("b5ad8bf8-34b7-498b-8cb8-952296238d96")
    parent = svchost.get_parent()

    svchost.get_children()
    parent.get_children()

    # parent.add_connections()

if __name__ == '__main__':
    main()