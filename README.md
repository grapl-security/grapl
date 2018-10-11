# grapl
Graph platform for Detection, Forensics, and Incident Response

Grapl aims to describe a network, and actions taking place on that network, as a graph.

The graph representation makes it easy to express complex attacker signatures
that span multiple discrete events. Automated contexting can be applied to
arbitrary signature matches by expanding the graph surrounding the match,
pulling in related information.

Grapl currently supports graph representations for:
* Process Start/ Stop
* File Create/Read/Write/Delete


### Example Use Case: Catching a malicious word macro

As an example, one can write a signature to catch a malicious word macro: 
* Process with `image_name` "word.exe" executes
* "word" executes a child process

```
{
  q(func: eq(image_name, "word")) 
  @filter(gt(create_time, 0) AND lt(create_time, 600))
  {
    uid, pid, create_time, image_name, terminate_time, node_key, asset_id
    children {
        expand(_all_),
    }
  }
}
```
(This is dgraph's query language, [graphql+](https://docs.dgraph.io/query-language/) - in the future a Python wrapper will be provided)

This could return a graph like:
![word_macro_hit](https://github.com/insanitybit/grapl/blob/master/images/word_child.png)


When these analyzers find matches, engagements are created. Engagements are a graph
representation of all of the events related to an incident. While a signature
might give us Word and the dropped payload, our engagement might pull in files
read by word, children of the 'payload' process, or other relevant information.

In the future it will be possible to interact with these engagements through
an API targeting Jupyter notebooks. For now the feature is limited to a visual
representation.

Grapl can automatically expand signature hits out to scope the engagement by
traversing the edges of the signature match, pulling relevant nodes into the
engagement.

Given the `word` and `payload` children we can recursively
add subsequent children, find the files read by word, etc.

![word_macro_graph](https://github.com/insanitybit/grapl/blob/master/images/word_macro_graph.png)

Even in cases where your detections are built on discrete events Grapl should
be able to provide benefits with its automated scoping.


### Current State

**Grapl is currently: Alpha Quality**

**What Works**
* Can parse process and file events, if they conform to the 'generic' parser
* Can identify and merge generated subgraphs into master graph
* Visualizing graphs via `dgraph-ratel`


**What Doesn't Work**
* No support for custom parsers
* Attributing ip addresses to assets
* Analyzer concept is immature and bulky
* Automated scoping of engagements is unreliable

Note that Grapl has not been given the security attention it deserves. I do not recommend
using it without examining the generated Cloudformation stack and source code.


### Next Steps

The immediate next steps are:
* Get the pipeline from subgraph generators to graph merging up to RC quality
    * Support arbitrary log parsers
    * Remove any hardcoded infra information
    * Handle a few edge cases that are currently left aside
* Re-architect the analyzer concept so that individual signatures don't map to
    individual lambdas, generally lower cost of writing analyzers
* Engagement creation and automated scoping
* Engagement interactions via Python API

Eventually I intend to support:
* Network relationships between nodes - ip, dns
* User and Asset nodes
* Much better/ higher level libraries for writing parsers and analyzers


## Architecture Diagram

Grapl has a lot of moving parts. This is the current architecture doc.

[grapl_arch](https://github.com/insanitybit/grapl/blob/master/images/grapl_arch.png)

As the diagram shows, Grapl is built primarily as a Pub Sub system. The goal is to make it easy to link
your own services up, move Grapl's own services around, and extend the platform to match your need.

Grapl is primarily built in Rust, with the Analyzers being built in Python.

## Setting up Grapl

### Building the binaries

In order to build the rust binaries for aws lambda you'll need to use the
[rust-musl-builder](https://github.com/emk/rust-musl-builder/) project.

[Due to a dependency on grpcio, you'll need to modify this project accordingly:](https://github.com/emk/rust-musl-builder/issues/53)

Due to the dockerized builds, library dependencies in grapl are expected to be copied into the service folder.

In the future I can package and provide built binaries, and with a bit of Docker work the need to copy dependencies
should go away as well.

### DGraph

Grapl relies on DGraph, and expects two **separate** dgraph instances. Grapl expects these
instances to be resolvable from the names:
`db.mastergraph`
`db.engagementgraph`


### Deploying

The majority of Grapl infrastructure is managed via [aws-cdk](https://gitter.im/awslabs/aws-cdk).

See `aws-cdk` docs for setup instructions.

Once the binaries are build you can zip them up, move them to the `grapl-cdk` folder, and deploy the stacks.


I recommend running `cdk diff` to see what resource changes you can expect.

```
cdk deploy vpcs-stack && \
cdk deploy event-emitters-stack && \
cdk deploy history-db-stack && \
cdk deploy generic-subgraph-generator-stack && \
cdk deploy node-identifier-stack && \
cdk deploy graph-merger-stack && \
cdk deploy word-macro-analyzer-stack && \
cdk deploy engagement-creation-service-stack
```
Your DGraph cluster security groups will need to allow traffic from the graph-merger, any analyzers,
and the engagement-creation-service.

In order to run the lambdas within their respective VPCs you may need to open a support request
with AWS to reserve extra Elastic IPs (20 has been sufficient for me).
