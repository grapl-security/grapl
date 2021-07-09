Both of these AMIs are heavily based off of 
https://github.com/hashicorp/terraform-aws-nomad/tree/master/examples/nomad-consul-ami
(but converted to HCL)

The difference between these two AMIs are:
- the nomad/consul config files that are provisioned by them.
- the AMI name prefix var

Really, that's it! They could totally be merged, but I'm leaving us open to
a potential future where they diverge more.
