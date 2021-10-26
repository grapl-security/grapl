# Queryables

Grapl provides powerful primitives for building graph based queries.

At the root of this query logic is the Queryable base class, though you
shouldn't ever have to work with that directly.

Queries are themselves Python classes that can be composed and constrained.

A simple query would look like this:

```python
ProcessQuery()
```

This query describes a process - any process, it's totally unconstrained.

We can execute this query in a few ways. Here are three examples,

```python
gclient = GraphClient()

all_processes = ProcessQuery().query(gclient)
one_process = ProcessQuery().query_first(gclient)
count = ProcessQuery().get_count(gclient)
```

## Queryable.query

Query the graph for all nodes that match.

`graph_client` - a GraphClient, which will determine which database to query
`contains_node_key` - a node_key that must exist somewhere in the query
`first` - return only the first `first` nodes. Defaults to 1000. When
`contains_node_key`, `first` is set to 1.

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

## Queryable.query_first

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

## Queryable.get_count

Query the graph, counting all matches.

`graph_client` - a GraphClient, which will determine which database to query
`first` - count up to `first`, and then stop.

`returns` - the number of matches for this query. If `first` is set, only count
up to `first`.

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

In this case, if our signature matches such that any of the nodes A, B, C, have
the node_key "node-key-to-query", we have a match - otherwise, no match.

## Boolean Logic

### And

For a single predicate constraint (with\_\* method) all constraints are
considered And'd.

This query matches a process name that contains both "foo" and "bar".

```python3
ProcessQuery()
.with_process_name(contains=["foo", "bar"])
```

### Or

Multiple predicate constraints are considered Or'd.

This query matches a process name that contains either "foo" or "bar".

```python3
ProcessQuery()
.with_process_name(contains="foo")
.with_process_name(contains="bar")
```

### Not

Any constraint can be wrapped in a Not to negate the constraint.

This query matches a process name that is _not_ "foo".

```python3
ProcessQuery()
.with_process_name(contains=Not("foo"))
```

### All Together

This query matches a process with a process*name that either is \_not* 'foo' but
ends with '.exe', _or_ it will match a process with a process containing "bar"
_and_ "baz".

```python3
ProcessQuery()
.with_process_name(contains=Not("foo"), ends_eith=".exe")
.with_process_name(contains=["bar", baz])
```

## Filters and functions

### with\_\* methods

Most Queryable classes provide a suite of methods starting with `with_*`.

For example, ProcessQuery provides a `with_process_name`.

[ProcessQuery.with_process_name](/nodes/process_node/#ProcessQuery)

```python
def with_process_name(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
    starts_with: Optional["StrCmp"] = None,
    regexp: Optional["StrCmp"] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> ProcessQuery:
    pass
```

The `process_name` field is indexed such that we can constrain our query
through:

### eq

Matches a node's `process_name` if it exactly matches `eq`

```python3
ProcessQuery().with_process_name(eq="svchost.exe")
```

### contains

Matches a node's `process_name` if it contains `contains`

```python3
ProcessQuery().with_process_name(contains="svc")
```

### ends_with

Matches a node's `process_name` if it ends with `ends_with`

```python3
ProcessQuery().with_process_name(ends_with=".exe")
```

### starts_with

Matches a node's `process_name` if it starts with `starts_with`

```python3
ProcessQuery().with_process_name(starts_with="svchost")
```

### regexp

Matches a node's `process_name` if it matches the regexp pattern `regexp`

```python3
ProcessQuery().with_process_name(regexp="svc.*exe")
```

### distance

Matches a node's `process_name` if it has a string distance of less than the
provided threshold

```python
ProcessQuery().with_process_name(distance=("svchost", 2))
```

### Example

Here's an example where we look for processes with a `process_name` that is
_not_ equal to `svchost.exe`, but that has a very close string distance to it.

```python3
ProcessQuery()
.with_process_name(eq=Not("svchost.exe"), distance=("svchost", 2))
```
