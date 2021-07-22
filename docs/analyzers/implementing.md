## Analyzers

Analyzers are the attack signatures that power Grapl's realtime detection logic.

Though implementing analyzers is simple, we can build extremely powerful and
efficient logic to catch all sorts of attacker behaviors.

### The Analyzer Base Class

To implement an Analyzer we must inherit from the Analyzer
[abstract base class](https://docs.python.org/3/library/abc.html).

```python
A = TypeVar("A", bound="Analyzer")

class Analyzer(abc.ABC):
    def __init__(self, dgraph_client: GraphClient) -> None:
        self.dgraph_client = dgraph_client

    @classmethod
    def build(cls: Type[A], dgraph_client: GraphClient) -> A:
        return cls(dgraph_client)

    @abc.abstractmethod
    def get_queries(self) -> OneOrMany[Queryable]:
        pass

    @abc.abstractmethod
    def on_response(self, response: Viewable, output: Any):
        pass
```

###### Analyzer.build

Returns an instance of your analyzer. This allows you to move dependency
management out of your `__init__`.

`cls` - the Class for your analyzer, which you should use for construction.
`graph_client` - an instance of a GraphClient

```python
@classmethod
def build(cls: Type[A], graph_client: GraphClient) -> A:
    return cls(dgraph_client)
```

###### Analyzer.get_queries

`get_queries` is where you define any of your graph signatures, either one or
multiple.

All queries returned must have the same type for the root node.

`returns` - all signatures to be matched against.

```python
@abc.abstractmethod
def get_queries(self) -> OneOrMany[Queryable]:
    pass
```

###### Analyzer.on_response

`on_response` is called if any of the sigantures from `get_queries` matched a
graph.

This method is where you can perform any subsequent logic that you couldn't fit
into your query, such as hitting an external threatfeed API, performing a count,
etc.

`response` - Guaranteed to be the Viewable type associated with the Queryable(s)
returned by `get_queries`

`output` - Provides a `send` method that takes an ExecutionHit

```python
@abc.abstractmethod
def on_response(self, response: Viewable, output: Any):
    pass
```

#### SuspiciousSvchost Example

Heres an example - we're going to write some logic to look for suspicious
executions of `svchost`.

```python
class SuspiciousSvchost(Analyzer):

    def get_queries(self) -> OneOrMany[ProcessQuery]:
        invalid_parents = [
            Not("services.exe"),
            Not("smss.exe"),
            Not("ngentask.exe"),
            Not("userinit.exe"),
            Not("GoogleUpdate.exe"),
            Not("conhost.exe"),
            Not("MpCmdRun.exe"),
        ]

        return (
            ProcessQuery()
            .with_process_name(eq=invalid_parents)
            .with_children(
                ProcessQuery().with_process_name(eq="svchost.exe")
            )
        )

    def on_response(self, response: ProcessView, output: Any):
        output.send(
            ExecutionHit(
                analyzer_name="Suspicious svchost",
                node_view=response,
                risk_score=75,
            )
        )
```

We've got a very straightforward Analyzer here. We don't need any custom `build`
or **init**, and our `on_response` contains no logic other than sending out an
ExecutionHit.

```python
    def get_queries(self) -> OneOrMany[ProcessQuery]:
        invalid_parents = [
            Not("services.exe"),
            Not("smss.exe"),
            Not("ngentask.exe"),
            Not("userinit.exe"),
            Not("GoogleUpdate.exe"),
            Not("conhost.exe"),
            Not("MpCmdRun.exe"),
        ]

        return (
            ProcessQuery()
            .with_process_name(eq=invalid_parents)
            .with_children(
                ProcessQuery().with_process_name(eq="svchost.exe")
            )
        )
```

The query is straightforward. We have a curated whitelist of parent processes
for svchost.exe.

Any process that is _not_ one of those is considered "invalid".

```python
    ProcessQuery() # Any Process
    .with_process_name(eq=invalid_parents)  # With an invalid parent process name
    .with_children(  # With any child processes
        ProcessQuery()
        .with_process_name(eq="svchost.exe")  # With the process name "svchost.exe".
    )
```

Our query is therefor read as: Any Process, with a process_name that exactly
matches `invalid_parents`, with any child process, where the child process_name
that exactly matches `svchost.exe`.

#### Adding Context

We may want to add some optional context to our query, without requiring that
context for our Analyzer to match. We can do this easily in our `on_response`
implenentation.

In the `on_response` method the `response` is going to be the root node of what
our query matched - in our case, this will be some invalid parent of
svchost.exe.

Some interesting context might be to get the binary path of that svchost.exe and
the parent process of our invalid_parent.

```python
    def on_response(self, response: ProcessView, output: Any):
        # Let's get the parent of our invalid_parent
        response.get_parent()

        # And the binary paths for any suspect child processes
        for child in response.children:
            if child.get_bin_file():
                child.bin_file.get_file_path()

        output.send(
            ExecutionHit(
                analyzer_name="Suspicious svchost",
                node_view=response,
                risk_score=75,
            )
        )
```

Unlike with the queries in
`get_queries', which have to be an exact match, our context is purely optional. We grab the information if it's available, but if it isn't we'll just move on.`

If the information is there we'll have so much more information when this
triggers, almost certainly enough to triage this without much investigation.
