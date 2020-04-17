# Local Grapl
In an effort to make Grapl even easier to get started with we’ve released a version that can run locally on your system! This post will outline through the process of setting up a local  Grapl environment, as well as performing a basic engagement to mimic an investigation.

**Pre-Requisites:**

Grapl requires the following dependencies. Before starting this tutorial, be sure the system you’re planning to run Grapl on has the following software installed:

- [docker](https://docs.docker.com/get-docker/)
- [docker-compose](https://docs.docker.com/compose/install/)


- A Python3 environment with:
    - [boto3](https://github.com/boto/boto3#quick-start) 
    - [zstd](https://pypi.org/project/zstd/)

Grapl has primarily been tested on Linux systems, where Docker support is best. If you’re working with another OS your experience may vary. If you do run into any problem, please file an issue or [let us know in our Slack channel](https://join.slack.com/t/grapl-dfir/shared_invite/zt-armk3shf-nuY19fQQuUnYk~dHltUPCw)!

**Running Grapl**

Getting Grapl set up on your system to run locally is a simple process! 

First, clone the Grapl repository, then run the command  `docker-compose up` in the directory where Grapl has been cloned. You may see warnings in your terminal as services boot up. Eventually the build process will reach a steady state - and shouldn’t take more than a few minutes!

```bash
git clone https://github.com/insanitybit/grapl.git
cd ./grapl/
docker-compose up
```

**Uploading Your Analyzer**

Next, we’ll upload a basic Analyzer (Grapl’s attack signatures), which searches for processes named `svchost` without a whitelisted parent process.

```python
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
    

To upload our Analyzer to Grapl, navigate to the root of the cloned `grapl` repository and run the following command: 

```bash
./upload_analyzer_local.sh
```

If you get an error similar to “could not connect to the endpoint URL”,  please give Grapl another minute to get started. 

**Adding Data to Grapl**

To get data into Grapl, please run the following command: 

```bash
python3 ./upload-sysmon-logs.py --bucket_prefix=local-grapl --logfile=eventlog.xml 
```

**Working With** **Grapl Data:** 

To analyze Grapl Data, open two browser windows in Google Chrome. 

In the first browser window, connect to the Grapl Notebook on [localhost:8888](http://localhost:8888). Credentials are not needed when running Grapl locally, just click the ‘submit’ button to get started. The Grapl Notebook is where we’ll interact with the engagements using Python.

Next, connect to the Engagement UX on [localhost:1234](http://localhost:1234). Log in with the password `graplpassword`. The Engagement UX displays risks in our environment. 

The lenses page will show one lens. A lens associates a risk with some kind of correlation point - in this case, an asset.


![](https://paper-attachments.dropbox.com/s_E2075176EF6E38314A4C35C9F7CAB863D54E38F5216C21919103EBEB3DB3EE0A_1586979707971_Screenshot+from+2020-04-15+12-40-08.png)


Click ‘link’ next to the lens score. Details relating to the first Lens will display in your browser!


![](https://paper-attachments.dropbox.com/s_E2075176EF6E38314A4C35C9F7CAB863D54E38F5216C21919103EBEB3DB3EE0A_1586979835729_Screenshot+from+2020-04-15+12-43-17.png)


 
Next, click the node labeled ‘cmd.exe’, and copy the value of `node_key`.
 
The  `Demo_Engagement` notebook creates a new engagement, which shows up on the ‘Lenses’ page. 

![](https://paper-attachments.dropbox.com/s_E2075176EF6E38314A4C35C9F7CAB863D54E38F5216C21919103EBEB3DB3EE0A_1586979716146_Screenshot+from+2020-04-15+12-40-29.png)


Click the ‘Run’ button three times and a new entry will appear on the Lenses page. This is our Engagement.


![](https://paper-attachments.dropbox.com/s_E2075176EF6E38314A4C35C9F7CAB863D54E38F5216C21919103EBEB3DB3EE0A_1586979770224_Screenshot+from+2020-04-15+12-42-38.png)


Copy the `node_key` for `cmd.exe` into the Enagement_Demo notebook. Then, paste the value in `<put cmd node_key here>`. Click `Run` to execute the cell.

Click the ‘link’ next to the ‘Demo’ Lens. The graph will update, indicating our node has been copied over.


![](https://paper-attachments.dropbox.com/s_E2075176EF6E38314A4C35C9F7CAB863D54E38F5216C21919103EBEB3DB3EE0A_1586979972077_Screenshot+from+2020-04-15+12-45-49.png)


As you continue to click the ‘run’ button in your Jupyter Notebook, the graph will update with new nodes and edges that get pulled into the Engagement graph.


![](https://paper-attachments.dropbox.com/s_E2075176EF6E38314A4C35C9F7CAB863D54E38F5216C21919103EBEB3DB3EE0A_1586980043595_Screenshot+from+2020-04-15+12-47-10.png)



![](https://paper-attachments.dropbox.com/s_E2075176EF6E38314A4C35C9F7CAB863D54E38F5216C21919103EBEB3DB3EE0A_1586980080696_Screenshot+from+2020-04-15+12-47-48.png)


 
As we pivot off of the data that we have, our graph expands to represent a ‘dropper’ behavior.

We’ve kept the data in our demo light so users to become familiar with Grapl’s core features.
