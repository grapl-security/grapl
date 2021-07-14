# Get the root amis field, which looks like:
# us-east-1:ami-111,us-east-2:ami-222,us-west-1:ami-333,us-west-2:ami-444
.builds[-1].artifact_id

# sample output:
# ["us-east-1:ami-111", ...]
| split(",")

# array of [region, ami-id]
# sample output: 
# [ ["us-east-1": "ami-111"], ...]
| map(split(":"))

# sample output:
# { "us-east-1": "ami-111", }
| reduce .[] as $region_and_ami_id(
    {};  # becomes `.` in below context
    .[$region_and_ami_id[0]] = $region_and_ami_id[1]  # {[k] = v}
)