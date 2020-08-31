# AWS setup
**NOTE that setting up Grapl *will* incur AWS charges! This can amount to hundreds of dollars a month based on the configuration. This setup script is designed for testing, and may include breaking changes in future versions, increased charges in future versions, or may otherwise require manually working with CloudFormation. 
If you need a way to set up Grapl in a stable, forwards compatible manner, please get in contact with me directly.**

Setting up a basic playground version of Grapl is pretty simple, though currently setup is only supported on Linux (setting up an Ubuntu EC2 instance is likely the easiest way to get access to a supported system).

## Installing Dependencies
To get started you'll need to install [npm](https://www.npmjs.com/), [typescript](https://www.typescriptlang.org/index.html#download-links), and the [aws-cdk](https://github.com/awslabs/aws-cdk#getting-started).

Your aws-cdk version should match the version in [Grapl's package.json file](https://github.com/insanitybit/grapl/blob/readmeupdate1/grapl-cdk/package.json#L29).

You'll also need to have local aws credentials, and a configuration profile. Instructions [here](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html)

If you intend to use Grapl's provided demo data, you'll allso need some Python3 dependencies.
- [boto3](https://github.com/boto/boto3)
- [zstd](https://pypi.org/project/zstd/)


## Clone, Configure, and Deploy
Grapl comes with binaries already in the repository.

Clone the repo:
`git clone https://github.com/insanitybit/grapl.git`
`cd ./grapl/grapl-cdk/`
`npm i # install dependencies`
`cdk boostrap # set up aws-cdk`

Add a `.env` file, and fill it in:

`BUCKET_PREFIX="<unique prefix to differentiate your buckets>"`

Run the deploy script
`./deploy_all.sh`

It will require confirming some changes to security groups, and will take a few minutes to complete.

This will give you a Grapl setup that’s adequate for testing out the service.

### Provisioning Grapl
At this point you need to provision the Graph databases and create a user. You can use the `Grapl Provision` notebook in this repo, and
the newly created 'engagement' notebook in your AWS account.

![](https://s3.amazonaws.com/media-p.slid.es/uploads/650602/images/6396963/Screenshot_from_2019-07-27_22-27-35.png)

Go to your AWS Sagemaker Console, open the Jupyter Notebook Grapl created for you, and upload the `Grapl Provision.ipynb` in this repository.

Run the notebook, and it will:
* Set up the schemas for your graph database
* Create a username, as well as a password, which you can use to log into your Grapl instance.

### Demo Data
You can send some test data up to the service by going to the root of the grapl repo and calling:
`python ./gen-raw-logs.py <your bucket prefix>`. 

*Note that this will likely impose charges to your AWS account.*

To use the Grapl UX you must navigate to the `index.html` in the grapl ux bucket.