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
counter.get_count_for(
    parent_process_name=p.process_name,
    child_process_name=p.children[0].process_name,
    excluding=p.node_key
)
```