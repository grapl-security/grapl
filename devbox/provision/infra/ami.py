import pulumi_aws as aws

import pulumi


def get_ami() -> pulumi.Output[aws.ec2.GetAmiResult]:
    # The default user for this AMI is, indeed, 'ubuntu'.
    # We propagate that information over to the GRAPL_DEVBOX_CONFIG for ssh.
    pulumi.export("devbox-user", "ubuntu")

    return aws.ec2.get_ami_output(
        owners=["099720109477"],  # Ubuntu / Canonical
        filters=[
            # the version of Ubuntu. We may want to upgrade this on a
            # semi-regular basis; note that would nuke the existing instance.
            aws.ec2.GetAmiFilterArgs(
                name="name",
                values=["*ubuntu-focal-20.04-amd64-server-20220404"],
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
        ],
    )
