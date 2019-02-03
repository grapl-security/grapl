import json
import pydgraph

# Look for processes with svchost.exe in their name with non services.exe parents
__example_analyzer_signature = """
  query q0($p: string)
    {
      q0(func: eq(node_key, "$p")) @cascade
      @filter(alloftext(image_name, "svchost"))
      {
        uid,
        ~children @filter(NOT alloftext(image_name, "services.exe")) {
        pid
        }
      }
    }
    """

def _analyzer(graph: Subgraph, sender: Connection):
    print('creating client')
    client_stub = pydgraph.DgraphClientStub('db.mastergraph:9080')
    client = pydgraph.DgraphClient(client_stub)
    print('created client')

    matches = []


    # TODO: Filter out non-process nodes
    for node_key in graph.subgraph.nodes:
        try:
            res = client.query(__example_analyzer_signature, {'$p': node_key})
        except Exception as e:
            print('analyzer query failed with: {}'.format(e))
            continue
        if not res:
            print('res was empty')
            continue

        res = json.loads(res.json)
        print('json response: {}'.format(res))

        [matches.push(sg) for sg in res['q0']]

    for match in matches:
        print('match {}'.format(match))

    sender.send(ExecutionComplete())



def analyzer(graph: Subgraph, sender: Connection):
    try:
        print('analyzing')
        _analyzer(graph, sender)
    except Exception as e:
        print('anaylzer failed: {}'.format(e))
        sender.send(None)
