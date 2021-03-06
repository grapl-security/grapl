# Provisioning Dgraph

Once the CDK deploy is complete you'll need to perform some additional
manual configuration to spin up the Dgraph cluster. The CDK deploy in
the previous section has created a Security Group and Instance Profile
for the EC2 Instances where we'll run the Docker Swarm cluster. It
only remains to spin up the EC2 instances, provision a Docker Swarm
cluster, and install Dgraph on it.

We'll use the `graplctl` tool to provision Dgraph:

1. Install `graplctl`. See the [graplctl
   README](https://github.com/grapl-security/grapl/tree/main/src/python/grapctl/README.md)
   for installation instructions.

2. Run the `graplctl dgraph create` command. Note that you will either
   need to configure the `GRAPL_REGION`, `GRAPL_DEPLOYMENT_NAME`, and
   `GRAPL_VERSION` environment variables or supply the corresponding
   values via the CLI options (`graplctl --help` for more
   information).
   For example:
   ```bash
   # Specify the flags if you didn't configure the environment variables.
   graplctl \
      --grapl-region your-region-1 \
      --grapl-deployment-name your-deployment-name \
      --grapl-version your-version-usually-latest \
      dgraph create \
        --instance-type i3.large
  ```

## DGraph operations

You can manage the DGraph cluster with the docker swarm tooling by
logging into one of the swarm manager with SSM. If you forget which
instances are the swarm managers, you can find them by running
`graplctl swarm managers`. For your convenience, `graplctl` also
provides an `exec` command you can use to run a bash command remotely
on a swarm manager. For example, to list all the nodes in the Dgraph
swarm you can run something like the following:

``` bash
graplctl swarm exec --swarm-id my-swarm-id -- docker node ls
```

If you forget which `swarm-id` is associated with your Dgraph cluster,
you may list all the swarm IDs in your deployment by running `graplctl
swarm ls`.
