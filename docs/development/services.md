# Services

## Network Diagram (Outdated)

![Network Diagram](network_diagram.png)

## Grapl Services - main pipeline

### Pipeline Ingress

**Input:** Receives an RPC to insert event logs (e.g. sysmon logs, osquery logs,
Cloudtrail logs). (Currently the plan is to allow one event/log per call, but we
may revisit this in the future.)

**Work:** We wrap those logs in an Envelope and throw it in Kafka. No transforms
are performed.

**Output:** Push logs to the 'raw-logs' topic for the next service, Generator
Dispatcher.

### Generator Dispatcher

**Input:** Pulls event logs from topic 'raw-logs'

**Work:** This service will:

- figure out which generators would respond to the incoming event-source
- call into Plugin Work Queue to enqueue that work in a durable Postgres store.

**Output:** Push this work to the Plugin Work Queue.

### Plugin Work Queue (for generators)

**Input:** Receives an RPC `push_execute_generator` to store generator work

**Work:** PWQ is a simple facade over a postgres database that lets us store and
manage work for a Generator plugin.

**Output:** Work is pulled from PWQ by Plugin Execution Sidecar.

## Plugin Execution Sidecar (for generators)

**Input:** Pulls work from Plugin Work Queue over gRPC

**Work:**

- Loop, querying for new work from PWQ
- When there is new work, delegate it to the Generator binary that runs
  alongside this sidecar over gRPC
- When the Generator's work is completed - successful or failure - call
  `acknowledge_generator` in PWQ. This will send the generator's output onto a
  Kafka queue.

**Output:** Put generated graphs on the 'generated-graphs' topic for the Node
Identifier.

### Plugin (generator)

**Input:** Receives an RPC containing an event log (i.e. a sysmon event)

**Work:** Turns these events into a standalone subgraph, independent of existing
Dgraph/Scylla state.

**Output:** The subgraph is returned as a response to the incoming RPC, going to
the Plugin Execution Sidecar.

### Node Identifier

**Input:** Pulls generated graphs from topic 'generated-graphs'

**Work:** Identifies nodes in the incoming subgraph against the canonical
identities of nodes in DynamoDB. The incoming nodes may be new, or they may
represent something already known in the master graph.

For instance, an incoming subgraph may refer to a process
`{PID:1234, name: "coolthing.exe", started at 1:00AM}`; it's possible that
Dgraph already has a node representing this exact same process. We'd like to
de-duplicate this process node.

**Output:** Push identified subgraphs to the 'identified-subgraph' topic for the
next service, Graph Merger.

### Graph Merger

**Input:** Pulls identified graphs from topic 'identified-graphs'

**Work:** Write the new edges and nodes from Node Identifier to Dgraph.

**Output:** Push merged graphs to the 'merged-graphs' topic for the next
service.

### TODO: Analyzer-dispatcher and analyzer subsystem

## Managerial RPC services

Services that

### Organization Management

TODO

### Plugin Registry

This service manages Generator and Analyzer plugins, letting one create, deploy
and teardown those plugins via RPC calls.

## Other services

### Model Plugin Deployer

TODO

### Event Source

Create, get and update an Event Source, which is an ID that lets us tie incoming
Generator work to which Plugin we think should process it.

## Other services

### Engagement View (aka UX)

Provides the main customer interaction with Grapl. This is not actually a
standalone service, but hosted as static assets inside Grapl Web UI.

### Graphql Endpoint

Graphql interface into our Dgraph database.

### Grapl Web UI

Provides authn/authz functions, and acts as a router to other services:

- Graphql Endpoint (/graphqlEndpoint)
- Model Plugin Deployer (current undergoing rewrite)

Also, hosts static assets like Engagement View.
