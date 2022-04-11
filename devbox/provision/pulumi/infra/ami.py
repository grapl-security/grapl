import pulumi
import pulumi_aws as aws

def get_ami() -> pulumi.Output[str]:
    aws.ec2.get_ami(
        owners=["099720109477"],  # Ubuntu / Canonical
        filters=[
            # the version of Ubuntu
            aws.ec2.GetAmiFilterArgs(
                name="name",
                values=["*/ubuntu-hirsute-21.04-amd64-server-20220118"],
            ),
            aws.ec2.GetAmiFilterArgs(
                name="architecture",
                values=["x86_64"],
            ),
            aws.ec2.GetAmiFilterArgs(
                name="root-device-type",
                values=["ebs"],
            ),
            aws.ec2.GetAmiFilterArgs(
                name="virtualization-type",
                values=["hvm"],
            ),
        ]
    )