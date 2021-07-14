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
# { "imgname.us-east-1": "ami-111", ...}
| reduce .[] as $region_and_ami_id(
    {};  # becomes the self - or `.` in below context
    # We use a period delimiter in the key so that, much later down the pipeline,  
    # we can shove it into `pulumi config set --path artifacts.imagename.region`
    .[$IMAGE_NAME + "." + $region_and_ami_id[0]] = $region_and_ami_id[1]  # {[k] = v}
)