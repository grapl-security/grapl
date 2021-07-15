#!/bin/bash

# Starts up a local docker registry.
# This is required for local nomad deployments

docker run -d -p 5000:5000 --name registry registry:2.7
