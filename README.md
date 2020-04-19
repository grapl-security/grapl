## Grapl

Grapl is a Graph Platform for Detection and Response with a focus on helping Detection Engineers and Incident Responders stop fighting their data and start connecting it. Grapl leverages graph data structures at its core to ensure that you can query and connect your data efficiently, model complex attacker behaviors for detection, and easily expand suspicious behaviors to encompass the full scope of an ongoing intrusion.

For a more in depth overview of Grapl, [read this](https://insanitybit.github.io/2019/03/09/grapl).

Essentially, Grapl will take raw logs, convert them into graphs, and merge those graphs into a Master Graph. It will then orchestrate the execution of your attack signatures, and provide tools for performing your investigations. Or watch [this talk at BSidesLV](https://www.youtube.com/watch?v=LjCtbpXQA9U&t=8028s) or [this talk at BSides San Francisco](https://www.youtube.com/watch?v=uErWRAJ4I4w) .

Grapl natively supports nodes for:

- Processes
- Files 
- Networking
- Plugin nodes, which can be used to arbitrarily extend the graph

and currently parses Sysmon logs or a generic JSON log format to generate these graphs.

Keep in mind that Grapl is not yet at a stable, 1.0 state, and is a fast moving project. Expect some minor bugs and breaking changes!

[Key Features](https://github.com/insanitybit/grapl#key-features)

[Setup](https://github.com/insanitybit/grapl#setup)

Questions? Try opening an issue in this repo, or joining the [Grapl slack channel (Click for invite)](https://join.slack.com/t/grapl-dfir/shared_invite/zt-armk3shf-nuY19fQQuUnYk~dHltUPCw).

## Key Features

**Identity**

If you’re familiar with log sources like Sysmon, one of the best features is that processes are given identities. Grapl applies the same concept but for any supported log type, taking psuedo identifiers such as process ids and discerning canonical identities.

Grapl then combines this identity concept with its graph approach, making it easy to reason about entities and their behaviors. Further, this identity property means that Grapl stores only unique information from your logs, meaning that your data storage grows sublinear to the log volume.

This cuts down on storage costs and gives you central locations to view your data, as opposed to having it spread across thousands of logs. As an example, given a process’s canonical identifier you can view all of the information for it by selecting the node.

![](https://d2mxuefqeaa7sj.cloudfront.net/s_7CBC3A8B36A73886DC59F4792258C821D6717C3DB02DA354DE68418C9DCF5C29_1553026555668_image.png)


**Analyzers**

Analyzers are your attacker signatures. They’re Python modules, deployed to Grapl’s S3 bucket, that are orchestrated to execute upon changes to grapl’s Master Graph.

Rather than analyzers attempting to determine a binary "Good" or "Bad" value for attack behaviors Grapl leverges a concept of Risk, and then automatically correlates risks to surface the riskiest parts of your environment.

Analyzers execute in realtime as the master graph is updated, using constant time operations. Grapl's Analyzer harness will automatically batch, parallelize, and optimize your queries. By leveraging constant time and sublinear operations Grapl ensures that as your organization grows, and as your data volume grows with it, you can still rely on your queries executing efficiently.

Grapl provides an analyzer library so that you can write attacker signatures using pure Python. See this [repo for examples](https://github.com/insanitybit/grapl-analyzers).

Here is a brief example of how to detect a suspicious execution of `svchost.exe`,
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
Keeping your analyzers in code means you can:

- Code review your alerts
- Write tests, integrate into CI
- Build abstractions, reuse logic, and generally follow best practices for maintaining software

Check out Grapl's [analyzer deployer plugin](https://github.com/insanitybit/grapl-analyzer-deployer) to see how you can keep your analyzers in a git repo that automatically deploys them upon a push to master.

**Engagements**

Grapl provides a tool for investigations called an Engagement. Engagements are an isolated graph representing a subgraph that your analyzers have deemed suspicious.

Using AWS Sagemaker hosted Jupyter Notebooks and Grapl's provided Python library you can expand out any suspicious subgraph to encompass the full scope of an attack.
As you expand the attack scope with your Jupyter notebook the Engagement Graph will update, visually representing the attack scope.

![](https://s3.amazonaws.com/media-p.slid.es/uploads/650602/images/6646682/Screenshot_from_2019-10-11_20-24-34.png)

**Event Driven and Extendable**

Grapl was built to be extended - no service can satisfy every organization’s needs. Every native Grapl service works by sending and receiving events, which means that in order to extend Grapl you only need to start subscribing to messages.

This makes Grapl trivial to extend or integrate into your existing services.

Grapl also provides a Plugin system, currently in beta, that allows you to expand the platforms capabilities - adding custom nodes and querying capabilities.

## Setup

NOTE that setting up Grapl *will* incur AWS charges! This can amount to hundreds of dollars a month based on the configuration. This setup script is designed for testing, and may include breaking changes in future versions, increased charges in future versions, or may otherwise require manually working with CloudFormation. 
If you need a way to set up Grapl in a stable, forwards compatible manner, please get in contact with me directly.

Setting up a basic playground version of Grapl is pretty simple, though currently setup is only supported on Linux (setting up an Ubuntu EC2 instance is likely the easiest way to get access to a supported system).

To get started you'll need to install [npm](https://www.npmjs.com/), [typescript](https://www.typescriptlang.org/index.html#download-links), and the [aws-cdk](https://github.com/awslabs/aws-cdk#getting-started).

Your aws-cdk version should match the version in [Grapl's package.json file](https://github.com/insanitybit/grapl/blob/readmeupdate1/grapl-cdk/package.json#L29).

You'll also need to have local aws credentials, and a configuration profile. Instructions [here](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html)

Clone the repo:

    git clone https://github.com/insanitybit/grapl.git

Change directories to the `grapl/grapl-cdk/` folder. There should already be build binaries.

Execute `npm i` to install the aws-cdk dependencies.

Execute `cdk bootstrap` to get cdk ready for your deployment.

Add a `.env` file, and fill it in:

    BUCKET_PREFIX="<unique prefix to differentiate your buckets>"

Run the deploy script
`./deploy_all.sh`

It will require confirming some changes to security groups, and will take a few minutes to complete.

This will give you a Grapl setup that’s adequate for testing out the service.

At this point you just need to provision the Graph databases and create a user. You can use the `Grapl Provision` notebook in this repo, and
the newly created 'engagement' notebook in your AWS account.

![](https://s3.amazonaws.com/media-p.slid.es/uploads/650602/images/6396963/Screenshot_from_2019-07-27_22-27-35.png)

Go to your AWS Sagemaker Console, open the Jupyter Notebook Grapl created for you, and upload the `Grapl Provision.ipynb` in this repository.

Run the notebook, and it will:
* Set up the schemas for your graph database
* Create a username, as well as a password, which you can use to log into your Grapl instance.


You can send some test data up to the service by going to the root of the grapl repo and calling:
`python ./gen-raw-logs.py <your bucket prefix>`. 

This requires the [boto3](https://github.com/boto/boto3) and [zstd](https://pypi.org/project/zstd/) Python modules.

*Note that this may impose charges to your AWS account.*

To use the Grapl UX you must navigate to the `index.html` in the grapl ux bucket.
https://<YOUR_BUCKET_PREFIX>engagement-ux-bucket.s3.amazonaws.com/index.html