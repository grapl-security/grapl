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

Next, we’ll upload a basic [Analyzer (Grapl’s attack signatures)](https://grapl.readthedocs.io/en/latest/analyzers/implementing.html), which searches for processes named "svchost" without a whitelisted parent process. We've provided a demo Analyzer in the Grapl repository. If you're interested in the code, see our Analyzer docs.

Grapl may take a couple of minutes to get started, so if you get an error similar to “could not connect to the endpoint URL”,  give Grapl a few more minutes to  finish provisioning. 
    

To upload our Analyzer to Grapl, navigate to the root of the cloned `grapl` repository and run the following command: 

```bash
cd etc/local_grapl/bin/
./upload_analyzer_local.sh
```

If you get an error similar to “could not connect to the endpoint URL”,  please give Grapl another minute to get started. 

**Adding Data to Grapl**

To get data into Grapl, please run the following command: 

```bash
cd etc/local_grapl/bin/
python3 ./upload-sysmon-logs.py --deployment_name=local-grapl --logfile=eventlog.xml 
```


**Working With Grapl Data:** 



To analyze Grapl Data, open two browser windows in Google Chrome. 

In the first window, navigate to the Grapl's Jupyter Notebook on localhost:8888.  The 'Grapl Notebook' is where we’ll interact with the engagements using Python. 

Log in with the password "graplpassword". Once logged in, you'll see a directory with files that will be used later in the tutorial.

![](https://static.wixstatic.com/media/aa91b3_2a9a44851cdf4ebb8703ae76af72b192~mv2.png/v1/fill/w_1480,h_455,al_c,q_90,usm_0.66_1.00_0.01/aa91b3_2a9a44851cdf4ebb8703ae76af72b192~mv2.webp)

***Logging In to Grapl Engagement UX:***
Navigate to localhost:1234, and enter the following credentials into the login form: 
Username: grapluser
Password: graplpassword

The Engagement UX displays risks in our environment. Credentials are not needed when running Grapl locally, just click the ‘submit’ button to get started! 

After logging in, you’ll be redirected to the Grapl UI. The Lenses section will show one lens which associates a risk with some kind of correlation point - in this case, an asset. 

To examine the graph of suspicious nodes and edges relating to our asset lens, click on the lens name, in this case ‘DESKTOP-FVSHABR0’. 

![](https://static.wixstatic.com/media/aa91b3_43750d8c9716482a8d8017d4826c93bf~mv2.png/v1/fill/w_1460,h_972,al_c,q_90/aa91b3_43750d8c9716482a8d8017d4826c93bf~mv2.webp)

After clicking the lens name, a graph will appear in the right panel. In this case, a graph with two nodes - "cmd.exe", "svchost.exe", and an edge between the two appears on the screen.

![](https://static.wixstatic.com/media/aa91b3_4ec6b529647e4310a7f79eb1788f35b4~mv2.png/v1/fill/w_1462,h_808,al_c,q_90/aa91b3_4ec6b529647e4310a7f79eb1788f35b4~mv2.webp)

Click the node labeled ‘cmd.exe’, and copy the value of node_key.

![](https://static.wixstatic.com/media/aa91b3_833b01debcfe4bbfa44e78d0bc1aba55~mv2.png/v1/fill/w_1464,h_756,al_c,q_90/aa91b3_833b01debcfe4bbfa44e78d0bc1aba55~mv2.webp)

The  Demo_Engagement notebook creates a new engagement, which shows up on the ‘Lenses’ page.  

Replace "<<put cmd node_key here>>" with the node key as a string.

![](https://static.wixstatic.com/media/aa91b3_31b92e85fedf4551918ed8147932d5d1~mv2.png/v1/fill/w_1480,h_748,al_c,q_90,usm_0.66_1.00_0.01/aa91b3_31b92e85fedf4551918ed8147932d5d1~mv2.webp)

Click the first block of code, then click the ‘Run’ button four times. A new lens will appear in the ‘Lenses’ list. This is our Engagement.

![](https://static.wixstatic.com/media/aa91b3_b8bd9fbf4c7f4e63b5a850a820423b35~mv2.png/v1/fill/w_1458,h_870,al_c,q_90/aa91b3_b8bd9fbf4c7f4e63b5a850a820423b35~mv2.webp)

As you continue to click the ‘run’ button in your Jupyter Notebook, the graph will update with new nodes and edges that get pulled into the Engagement graph.

![](https://static.wixstatic.com/media/aa91b3_d4540e548fbe42139af7e6eacb341364~mv2.png/v1/fill/w_1462,h_778,al_c,q_90/aa91b3_d4540e548fbe42139af7e6eacb341364~mv2.webp)

As we pivot off of the data that we have, our graph expands to visually display a‘dropper’ behavior.

![](https://static.wixstatic.com/media/aa91b3_a8edd9fb0c8c470480ced49373c9d53d~mv2.png/v1/fill/w_1460,h_1392,al_c,q_90/aa91b3_a8edd9fb0c8c470480ced49373c9d53d~mv2.webp)

We’ve kept the data in our demo light so users to become familiar with Grapl’s core features, but you can keep expanding the graph using the notebook to get the full story of what the attacker did.

Check out [our docs](https://grapl.readthedocs.io/en/latest/) to see other ways to interact with your data.



### What's Next?

Grapl is drastically improving in many ways. Recently we’ve undergone a full rewrite of our front-end experience, we're actively working to support more data sources, and improving documentation. 

To support these changes, we’ve expanded our team size, and are planning to grow quickly, so expect a significant acceleration in our development! We’ve hired multiple new engineers, who have either started or will start full-time with Grapl in the coming weeks.

We’ll have more exciting updates to share soon, keep an eye out for more improvements to Grapl by follow us [@GraplSec](https://twitter.com/graplsec) or join us on [Slack](https://join.slack.com/t/grapl-dfir/shared_invite/zt-armk3shf-nuY19fQQuUnYk~dHltUPCw)!