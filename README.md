# grapl_analyzerlib
Analyzer library for Grapl

This library provides two main constructs,

Queries and Entities.

Queries are used for performing subgraph searches against
Grapl's Master or Engagement graphs.

Entities are designed for pivoting off of query results.

https://pypi.org/project/grapl-analyzerlib/

`pip install grapl_analyzer`


### Querying
```python
from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzerlib.counters import ParentChildCounter
from grapl_analyzerlib.entities import ProcessQuery
from grapl_analyzerlib.entity_queries import Not

mclient = DgraphClient(DgraphClientStub('alpha0.mastergraphcluster.grapl:9080'))

# Query for suspicious svchost.exe's, based on the parent process not being whitelisted
p = (
    ProcessQuery()
    .with_process_name(contains=[
        Not("services.exe"),
        Not("lsass.exe"),
    ])
    .with_children(
        ProcessQuery()
        .with_process_name(ends_with="svchost.exe")
    )
    .query_first(mclient)
)
if p:
    # We now have a ProcessView, representing a concrete subgraph
    print(f"Found: {p.process_name} at path: {p.get_bin_file()}")

```


### Entity Pivoting
Given an entity `p`, such as from the above example.

```python
parent = p.get_parent()
siblings = parent.get_children()
bin_file = p.get_bin_file()
bin_file_creator = bin_file.get_creator()
```

We can easily pivot across the graph, incrementally expanding
the scope.

### Counting
Counters are currently provided as simple, specialized helpers.

Given an entity `p`, such as from the above example.

```python
counter = ParentChildCounter(mclient)
count = counter.get_count_for(
    parent_process_name=p.process_name,
    child_process_name=p.children[0].process_name,
    excluding=p.node_key
)

if count <= Seen.Once:
    print("Seen one time or never")
```
