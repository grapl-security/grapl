# grapl
Graph platform for Detection, Forensics, and Incident Response

Grapl aims to describe a network, and actions taking place on that network,
as a graph. By doing so it will make querying for interconnected
events efficient and easily expressed while allowing for automated scoping
of your investigations.

Grapl currently supports graph representations for:
* Process Start/ Stop
* File Create/Read/Write/Delete


### Example Use Case

As an example, one can write signatures a malicious word macro: 
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

This could return a graph like:
![word_macro_hit](https://github.com/insanitybit/grapl/blob/master/images/word_child.png)


When these analyzers find matches, engagements are created. In the future
it will be possible to interact with these engagements through an API targeting
Jupyter notebooks. For now the feature is limited to a visual representation.

Grapl can automatically expand signature hits out to scope our engagement by
traversing the edges of the signature match, pulling more nodes into the
engagement.

Given the `word` and `payload` children we can recursively
add subsequent children, find the files read by word, etc.

![word_macro_graph](https://github.com/insanitybit/grapl/blob/master/images/word_macro_graph.png)


Even in cases where your detections are built on discrete events Grapl should
be able to provide benefits with its automated scoping.


### Features

Grapl consists primarily of:

1. Parsers to turn logs into subgraphs, and infra to merge those subgraphs into a master graph
2. Analyzers to query the master graph in realtime
3. Engagements to interact with the output of analyzers

With 'behind the scenes' features like automated scoping,
attribution of ips to assets, etc.

### Current State

**Grapl is currently: Alpha Quality**

Currently the majority of the pipeline from parsing to population
of the master graph is working fairly well. Grapl only supports a specific set
of JSON encoded logs for Processes and Files.

The analysis and engagement pieces are fragile and unreliable. As an example,
there is no whitelisting, and automated scoping of the engagement is in a
broken state.

Setting Grapl up requires some source code modifications due to hardcoded
resources.

Grapl exposes secrets, such as the history database username + password,
and otherwise has not been given the security attention it deserves.

Building the rust binaries requires a custom docker image and a nightly
compiler (nightly is mostly unnecessary and I can eventually target stable).

### Next Steps

The immediate next steps are:
* Get the pipeline from subgraph generators to graph merging up to RC quality
* Re-architect the analyzer concept so that individual signatures don't map to
    individual lambdas
* Engagement creation and automated scoping
* Engagement interactions via Python API

Eventually, I intend to support:
* Network relationships between nodes - ip, dns
* User and Asset nodes


## Setting up Grapl

### Building the binaries

In order to build the rust binaries for aws lambda you'll need to use the
[rust-musl-builder](https://github.com/emk/rust-musl-builder/) project.

[Due to a dependency on grpcio, you'll need to modify this project accordingly:](https://github.com/emk/rust-musl-builder/issues/53)

Due to the dockerized builds, library dependencies in grapl are expected to be copied into the service folder.

In the future I can package and provide built binaries, and with a bit of Docker work the need to copy dependencies
should go away as well.

### DGraph

Grapl relies on DGraph, and expects two separate dgraph instances. Grapl expects these
instances to be resolvable from the names:
`db.mastergraph`
`db.engagementgraph`


### Deploying

The majority of Grapl infrastructure is managed via [aws-cdk](https://gitter.im/awslabs/aws-cdk).

See `aws-cdk` docs for setup instructions.

Once the binaries are build you can zip them up, move them to the `grapl-cdk` folder, and `cdk deploy grapl-cdk`.

But this is not enough. For one thing the `grapl-stack` is large, comprising more than 200
resources, which is more than CloudFormation allows. As such you'll have to pin a few things
up manually. I haven't spent time splitting the stack up (contributions very welcome).

Security groups will need to be modified for access to the `historydb` and dgraph clusters.