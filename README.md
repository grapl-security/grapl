## Grapl

Grapl is a Graph Platform for Detection and Response. 

For a more in depth overview of Grapl, [read this](https://insanitybit.github.io/2019/03/09/grapl).

In short, Grapl will take raw logs, convert them into graphs, and merge those graphs into a Master Graph. It will then orchestrate the execution of your attack signatures and provide tools for performing your investigations.

Grapl supports nodes for:

- Processes (Beta)
- Files (Beta)
- Networking (Alpha)

and currently parses Sysmon logs or a generic JSON log format to generate these graphs.

## Key Features

**Identity**

If you’re familiar with log sources like Sysmon, one of the best features is that processes are given identities. Grapl applies the same concept but for any supported log type, taking psuedo identifiers such as process ids and discerning canonical identities.

This cuts down on storage costs and gives you central locations to view your data, as opposed to having it spread across thousands of logs. As an example, given a process’s canonical identifier you can view all of the information for it by selecting the node.

![](https://d2mxuefqeaa7sj.cloudfront.net/s_7CBC3A8B36A73886DC59F4792258C821D6717C3DB02DA354DE68418C9DCF5C29_1553026555668_image.png)


**Analyzers (Beta)**

Analyzers are your attacker signatures. They’re Python modules, deployed to Grapl’s S3 bucket, that are orchestrated to execute upon changes to grapl’s Master Graph.

Analyzers execute in realtime as the master graph is updated.

Grapl provides an analyzer library (alpha) so that you can write attacker signatures using pure Python:

```python
    def signature_graph(node_key: str) -> str:
        child = Process() \
            .with_image_name(contains="svchost.exe") \
            .with_node_key(eq=node_key)
    
        parent = Process() \
            .with_image_name(contains=Not("services.exe"))
        return parent.with_child(child).to_query()
```
Keeping your analyzers in code means you can:

- Code review your alerts
- Write tests, integrate into CI
- Build abstractions, reuse logic, and generally follow best practices for maintaining software

**Engagements (alpha)**

Grapl provides a tool for investigations called an Engagement. Engagements are an isolated graph representing a subgraph that your analyzers have deemed suspicious.

Using AWS Sagemaker hosted Jupyter Notebooks, Grapl will (soon) provide a Python library for interacting with the Engagement Graph, allowing you to pivot quickly and maintain a record of your investigation in code.


![](https://d2mxuefqeaa7sj.cloudfront.net/s_7CBC3A8B36A73886DC59F4792258C821D6717C3DB02DA354DE68418C9DCF5C29_1553037156946_file.png)


There is no UI for the engagements yet but I hope to build one soon - a live updating view of the engagement graph as you interact with it in the notebook.

**Event Driven and Extendable**

Grapl was built to be extended - no service can satisfy every organization’s needs. Every native Grapl service works by sending and receiving events, which means that in order to extend Grapl you only need to start subscribing to messages.

This makes Grapl trivial to extend or integrate into your existing services.

![](https://d2mxuefqeaa7sj.cloudfront.net/s_7CBC3A8B36A73886DC59F4792258C821D6717C3DB02DA354DE68418C9DCF5C29_1553040182040_file.png)



![](https://d2mxuefqeaa7sj.cloudfront.net/s_7CBC3A8B36A73886DC59F4792258C821D6717C3DB02DA354DE68418C9DCF5C29_1553040197703_file.png)


## Setup

Setting up a basic playground version of Grapl is pretty simple. 

Clone the repo:

    git clone https://github.com/insanitybit/grapl.git

Change directories to the `grapl/grapl-cdk/` folder. There should already be build binaries.

Add a `.env` file, and fill it in:

    HISTORY_DB_USERNAME=username
    HISTORY_DB_PASSWORD=password
    BUCKET_PREFIX="<unique prefix to differentiate your buckets>"
    GRAPH_DB_KEY_NAME=<name of SSH key, if debug mode is enabled, to SSH to graphdb>

Run the deploy script
`./deploy_all.sh`

You’ll then need to [set up dgraph](https://docs.dgraph.io/deploy/) on the two EC2 instances that have been set up for you.

This will give you a Grapl setup that’s adequate for testing out the service.

