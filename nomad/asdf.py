import pydgraph

# client_stub = pydgraph.DgraphClientStub('192.168.1.20:20052')
client_stub = pydgraph.DgraphClientStub('localhost:9080')
client = pydgraph.DgraphClient(client_stub)
print(client.check_version())

