

## Queryables
Grapl provides powerful primitives for building graph based queries.

At the root of this query logic is the Queryable base class, though
you shouldn't ever have to work with that directly.

Queries are themselves Python classes that can be composed and constrained.

A simple query would look like this:
```python
ProcessQuery()
```

This query describes a process - any process, it's totally unconstrained.

We can execute this query in a few ways. Here are three examples,
```python
mclient = MasterGraphClient()
    
all_processes = ProcessQuery().query(mclient)
one_process = ProcessQuery().query_first(mclient)
count = ProcessQuery().get_count(mclient)    
```

###### Queryable.query

Query the graph for all nodes that match.

`graph_client` - a GraphClient, which will determine which database to query
`contains_node_key` - a node_key that must exist somewhere in the query
`first` - return only the first `first` nodes. Defaults to 1000. 
          When `contains_node_key`, `first` is set to 1.

`returns` - a list of nodes that matched your query

```python
    def query(
        self,
        graph_client: GraphClient,
        contains_node_key: Optional[str] = None,
        first: Optional[int] = 1000,
    ) -> List["NV"]:
        pass
```


###### Queryable.query_first
Query the graph for the first node that matches.

`graph_client` - a GraphClient, which will determine which database to query
`contains_node_key` - a node_key that must exist somewhere in the query

`returns` - a list of nodes that matched your query
```python
    def query_first(
        self, 
        graph_client: GraphClient, 
        contains_node_key: Optional[str] = None
    ) -> Optional["NV"]:
        pass
```

###### Queryable.get_count
Query the graph, counting all matches. 

`graph_client` - a GraphClient, which will determine which database to query
`first` - count up to `first`, and then stop.

`returns` - the number of matches for this query. If `first` is set, only count up to `first`.
```python
    def get_count(
        self,
        graph_client: GraphClient,
        first: Optional[int] = None,
    ) -> int:
        pass
```


### contains_node_key

In some cases, such as when writing Grapl Analyzers, we want to execute a query
where a node's node_key may be anywhere in that graph.

For example,

```python
query = (
    ProcessQuery()  # A
    .with_bin_file(
        FileQuery()  # B
        .with_spawned_from(
            ProcessQuery()  # C
        )
    )
)

query.query_first(mclient, contains_node_key="node-key-to-query")

```

In this case, if our signature matches such that any of the nodes A, B, C, have the node_key
"node-key-to-query", we have a match - otherwise, no match.