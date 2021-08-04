# Implementing a Graph Generator

Graph Generators are Grapl's parser services; they take in raw events and they
produce a graph representation.

As an example, a geneartor for OSQuery process_event table would take in an
event like this:

```json
{
  "action": "added",
  "columns": {
    "uid": "0",
    "time": "1527895541",
    "pid": "30219",
    "path": "/usr/bin/curl",
    "auid": "1000",
    "cmdline": "curl google.com",
    "ctime": "1503452096",
    "cwd": "",
    "egid": "0",
    "euid": "0",
    "gid": "0",
    "parent": "30200"
  },
  "unixTime": 1527895550,
  "hostIdentifier": "vagrant",
  "name": "process_events",
  "numerics": false
}
```

And produce a graph that represents the entities and relationships in the event.

For example, we might have a graph that looks like this (minimally):

```

// A node representing the child process
ChildProcessNode {
    pid: event.columns.pid,  // The child process pid
    created_timestamp: event.columns.time  // The child process creation time
}

// A node representing the parent
ParentProcessNode {
    pid: event.columns.parent,  // The parent process pid
    seen_at_timestamp: event.columns.time  // The time that we saw the parent process
}

// An edge, relating the two processes
ChildrenEdge {
    from: ParentProcess,
    to: ChildProcess,
}

```

The goal of this document is to guide you through how to build that function.

## Getting starting

First off, Grapl's graph generators are currently written in the Rust
programming language. There are a number of benefits to using Rust for parsers,
such as it's high performance while retaining memory safety.

Don't be intimidated if you don't know Rust! You don't have to be an expert to
write a generator.

### Installing Requirements

You can install rust by running this script:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Creating the Generator Project

```bash
cargo new our-graph-generator
cd ./out-graph-generator/
```

Modify the `Cargo.toml` to include our Grapl generator library:

```toml
[dependencies]
graph-generator-lib = "*"

```

This library will provide the primitives we need in order to parse our data into
a graph.

### Implementing the EventHandler

Grapl's going to handle all of the work to get data in and out of your function,
all you need to do is add the entrypoint and implement an interface to do the
parsing.

The interface is called the EventHandler.

### Testing With Local Grapl
