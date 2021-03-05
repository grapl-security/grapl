# AWS setup
**NOTE that setting up Grapl *will* incur AWS charges! This can amount to hundreds of dollars a month based on the configuration.**
This setup script is designed for testing, and may include breaking changes in future versions, increased charges in future versions, or may otherwise require manually working with CloudFormation. 
If you need a way to set up Grapl in a stable, forwards compatible manner, please get in contact with us directly.

## Installing Dependencies
To get started, you'll need to install the following dependencies:

- Node
- Typescript
- AWS CDK: `npm i -g aws-cdk@X.Y.Z` 
  - version must be >= the version in [Grapl's package.json file](https://github.com/grapl-security/grapl/blob/master/src/js/grapl-cdk/package.json) - for instance, `@1.71.0`
- AWS CLI: `pip install awscli`

You'll also need to have local AWS credentials, and a configuration profile. Instructions [here](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html).

If you intend to use Grapl's provided demo data, you'll allso need some Python3 dependencies.
- [boto3](https://github.com/boto/boto3)
- [zstd](https://pypi.org/project/zstd/)


## Clone from repo
First things first, clone the repo:
```bash
git clone https://github.com/grapl-security/grapl.git
cd grapl/src/js/grapl-cdk/
```

## Build deployment artifacts

To execute a local Grapl build, run the following in Grapl's root:

```bash
TAG=$GRAPL_VERSION make zip
```

`GRAPL_VERSION` can be any name you want. Just make note of it, we'll
use it in the next step.

Alternatively, you can set TAG in a file named `.env` in the Grapl root directory. Ex:

```bash
$ cat .env
TAG="v0.0.1-example"
$ make zip
```

Similar can be done for the environment variables corresponding to CDK
deployment parameters documented in the following section.

Your build outputs should appear in the `src/js/grapl-cdk/zips/` directory.

## Configure
There are a few CDK deployment parameters you'll need to configure before you can deploy. 
Each of these can be found in `bin/deployment_parameters.ts`:

1. `deployName` (required)

    Name for the deployment to AWS. We recommend prefixing the
    deployment name with "Grapl-" to help identify Grapl resources in
    your AWS account, however this isn't necessary.

    Note: This name must be globally (AWS) unique, as names for AWS S3
    buckets will be dervied from this.

    env: `GRAPL_DEPLOYMENT_NAME`

2. `graplVersion`

    The version of Grapl to deploy. This string will be used to look
    for the appropriate filenames in the `zips/` directory.

    Defaults to `latest`.

    env: `GRAPL_VERSION`

3. `watchfulEmail` (optional)

    Setting this enables [Watchful](https://github.com/eladb/cdk-watchful) for
    monitoring Grapl with email alerts.

    env: `GRAPL_CDK_WATCHFUL_EMAIL`

4. `operationalAlarmsEmail` (optional)

    Setting this enables alarms meant for the operator of the Grapl stack.

    env: `GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL`

5. `securityAlarmsEmail` (optional)

    Setting this enables alarms meant for the consumer of the Grapl
    stack, for example, "a new risk node has been found".

    env: `GRAPL_CDK_SECURITY_ALARMS_EMAIL`

Alternatively, these can be set via the environment variables
mentioned for each above. The environment variables take precedence
over the values in `bin/deployment_parameters.ts`.

When deploying to production we recommend *not* using environment
variables for setting parameters, but rather set them in
`bin/deployment_parameters.ts` and save the changes in a git
branch. This should help future maintenance of the deployment.

## Deploy CDK
To deploy Grapl with the CDK, execute the following from the `src/js/grapl-cdk` directory.

```bash
npm i
npm run build
env CDK_NEW_BOOTSTRAP=1 cdk bootstrap \
  --cloudformation-execution-policies arn:aws:iam::aws:policy/AdministratorAccess

# This last step should take a while - roughly an hour.
./deploy_all.sh
```

If you have configured an email address for Watchful (see previous
section) you should receive an email with subject *"AWS Notification -
Subscription Confirmation"* containing a link to activate the
subscription. Click the link to begin receiving CloudWatch alerts.

## Provisioning Dgraph
Next, we need to spin up some EC2 instances that will host the graph database, Dgraph.

[Follow the instructions here.](./dgraph_provision)

## Provisioning Grapl
At this point you need to provision the Graph databases and create a user. 
- Go to the AWS Console
- Open AWS Sagemaker from the Services list
- Click 'Notebook Instances' on the left bar
- Click 'Open Jupyter' next to the single notebook
- Finally, hit the 'Upload' button and navigate to `$GRAPL_REPO_ROOT/etc/Grapl\ Provision.ipynb`

You should be presented with a view something like this:

![](https://s3.amazonaws.com/media-p.slid.es/uploads/650602/images/6396963/Screenshot_from_2019-07-27_22-27-35.png)


Run each cell in the notebook, and it will:
* Set up the schemas for your graph database
* Create a username + password, which you can use to log into your Grapl instance.
  * Hide this password somewhere safe - it's the only time we'll give it to you!

### Demo Data
You can send some test data up to the service by going to the root of the grapl repo and calling:
```bash
cd $GRAPL_ROOT

# upload analyzers
etc/aws/upload_analyzer_prod.sh

# upload logs
AWS_REGION=$GRAPL_REGION \ 
python3 etc/local_grapl/bin/upload-sysmon-logs.py \
  --deployment_name $GRAPL_DEPLOYMENT_NAME \
  --logfile etc/sample_data/eventlog.xml 
```

*Note that this will likely impose charges to your AWS account.*

You can then view the progress of this data flowing through your deployment
by looking at the Cloudwatch Dashboard named 
`{GRAPL_DEPLOYMENT_NAME}-PipelineDashboard`.

### Accessing the Grapl UX (Engagement Edge)
You can find the base url in `src/js/grapl-cdk/cdk-output.json`; just append
a `/index.html` to the URL in that file.
