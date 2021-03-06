import pulumi

# Use this to modify behavior or configuration for provisioning in
# Local Grapl (as opposed to any other real deployment)
IS_LOCAL = pulumi.get_stack() == "local-grapl"
