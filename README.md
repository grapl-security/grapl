# grapl
Graph platform for Detection, Forensics, and Incident Response

Grapl aims to describe a network, and actions taking place on that network,
as a graph. This makes querying for interconnected
events efficient, easily expressed, and allows you to automate scoping
your investigations.

Currently supported graph representations:
* Process Start/ Stop
* File Create/Read/Write/Delete


### Example Use Case: Writing a signature to catch a malicious word macro

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


When these analyzers find matches, engagements are created and displayed in a visual representation. In the future
it will be possible to interact with these engagements through an API targeting
Jupyter notebooks.

Grapl can automatically expand signature hits out to scope our engagement by
traversing the edges of the signature match, pulling more nodes into the
engagement.

Given the `word` and `payload` children we can recursively
add subsequent children, find the files read by word, etc.

![word_macro_graph](https://github.com/insanitybit/grapl/blob/master/images/word_macro_graph.png)


Even in cases where your detections are built on discrete events Grapl should
be able to provide benefits with its automated scoping.


### Current State

**Grapl is currently: Alpha Quality**

**What Works**
* Can parse process and file events, if they conform to our 'generic' parser
* Can identify and merge generated subgraphs into master graph


**What Doesn't Work**
* No support for custom parsers
* Analyzer concept is immature and bulky
* No whitelisting for analyzers yet
* Automated scoping of engagements is totally broken
* Build/ Deploy is overly manual, very fragile
* Grapl is unoptimized, I have spent virtually 0 time optimizing it except what was necessary
    to get it working. There's a ton of low hanging fruit. 

Note that Grapl exposes secrets, such as the history database username + password,
and otherwise has not been given the security attention it deserves. I do not recommend
using it without examining the generated Cloudformation stack and source code.


### Next Steps

The immediate next steps are:
* Get the pipeline from subgraph generators to graph merging up to RC quality
    * Support arbitrary log parsers
    * Remove any hardcoded infra information/ sensitive information
    * Handle a few edge cases that are currently left aside
* Re-architect the analyzer concept so that individual signatures don't map to
    individual lambdas
* Engagement creation and automated scoping
* Engagement interactions via Python API

Eventually, I intend to support:
* Network relationships between nodes - ip, dns
* User and Asset nodes
* Much better/ higher level libraries for writing parsers and analyzers


## Architecture Diagram

Grapl has a lot of moving parts. This is the current architecture doc.

Note that this doc does not include every service, and includes some that have yet to be built.

![grapl_arch](https://github.com/insanitybit/grapl/blob/master/images/grapl_arch.png)


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
