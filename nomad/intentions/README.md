# Consul Intentions

Consul Intentions are essentially a service to service firewall. It supports
both L4 routing (all or nothing traffic between services) and more fine-grained
L7 routing (allow only traffic with the right path and REST verb). Intentions
will be required when we add a deny-by-default acl.

We write these config files in JSON format. These are currently deployed
dynamically with pulumi, but can also be deployed as files to a consul agent's
data directory if necessary (for example if the UI is no longer accessible but
SSH is)

While hcl format is also supported, we're using JSON since JSON is better
defined and we won't need to pull in another serialization library, which could
be subject to vulnerabilities.

## References

- https://www.consul.io/docs/connect/config-entries/service-intentions
