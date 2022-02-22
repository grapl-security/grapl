# Deploying Analyzers

Once you've written your Analyzers you'll want to deploy them to Grapl.

To upload the analyzer file to the bucket where it lives, use the 
`graplctl upload analyzer` tool, which works both locally and with AWS.

## Deploying from Github

We can keep our detection logic in Github, which will allow us to perform code
reviews, linting, and automate the deployment of our analyzers.

As an example,
[insanitybit/grapl-analyzers](https://github.com/insanitybit/grapl-analyzers) is
set up to use this webhook.