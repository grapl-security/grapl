import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as iam from '@aws-cdk/aws-iam';

export interface SwarmProps {

    // The VPC where the Docker Swarm cluster will run
    readonly vpc: ec2.IVpc;

    // The service-specific (e.g. DGraph) ports to open internally
    // within the Docker Swarm cluster.
    readonly internalServicePorts: ec2.Port[];
}

export class Swarm extends cdk.Construct {
    private readonly swarmSecurityGroup: ec2.ISecurityGroup;

    constructor(
        scope: cdk.Construct,
        id: string,
        swarmProps: SwarmProps
    ) {
        super(scope, id);

        const swarmSecurityGroup = new ec2.SecurityGroup(scope, id + "-swarm-security-group", {
            vpc: swarmProps.vpc,
            allowAllOutbound: false
        });
        // allow the bastion machine to make outbound connections to
        // the Internet for these services:
        //   TCP 443 -- AWS SSM Agent (for handshake)
        //   TCP 80 -- yum package manager and wget (install docker-machine)
        swarmSecurityGroup.connections.allowToAnyIpv4(ec2.Port.tcp(443));
        swarmSecurityGroup.connections.allowToAnyIpv4(ec2.Port.tcp(80));

        // allow hosts in the swarm security group to communicate
        // internally on the following ports:
        //   TCP 22 -- SSH
        //   TCP 2376 -- secure docker client (docker-machine)
        //   TCP 2377 -- inter-node communication (only needed on manager nodes)
        //   TCP + UDP 7946 -- container network discovery
        //   UDP 4789 -- overlay network traffic
        swarmSecurityGroup.connections.allowInternally(ec2.Port.tcp(22));
        swarmSecurityGroup.connections.allowInternally(ec2.Port.tcp(2376));
        swarmSecurityGroup.connections.allowInternally(ec2.Port.tcp(2377));
        swarmSecurityGroup.connections.allowInternally(ec2.Port.tcp(7496));
        swarmSecurityGroup.connections.allowInternally(ec2.Port.udp(7496));
        swarmSecurityGroup.connections.allowInternally(ec2.Port.udp(4789));

        // allow hosts in the swarm security group to communicate
        // internally on the given service ports.
        swarmProps.internalServicePorts.forEach(
            (port, _) => swarmSecurityGroup.connections.allowInternally(port)
        );

        this.swarmSecurityGroup = swarmSecurityGroup;

        const bastion = new ec2.BastionHostLinux(this, id + 'bastion', {
            vpc: swarmProps.vpc,
            securityGroup: swarmSecurityGroup,
            instanceType: new ec2.InstanceType("t3a.nano"),
            instanceName: "SwarmBastion"
        });

        /* configure a bunch of AWS permissions to enable
         * docker-machine to do things with instances.
         *
         * with this set of permissions, the docker-machine invocation
         * requires the following parameters passed on the command
         * line, else it won't work:
         *
         * --amazonec2-vpc-id <vpc_id>
         * --amazonec2-security-group <security_group>
         * --amazonec2-keypair-name <keypair_name>
         * --amazonec2-ssh-keypath <ssh_keypath>
         *
         * this seems like it's a fairly locked-down configuration, yet
         * flexible enough to allow interesting use-cases (e.g. spot
         * instances).
         *
         * see this github issue comment for more info:
         *
         * https://github.com/docker/machine/issues/1655#issuecomment-409407523
         *
         * "DescribeSecurityGroups" -- required to check whether the
         * --amazonec2-security-group actually exists
         *
         * "CreateSecurityGroup" -- not sure why this is required
         *
         * "DescribeSubnets" -- required to find the subnet
         *
         * "DescribeKeyPairs" -- to validate whether the keypair
         * actually exists
         *
         * "CreateKeyPair" -- not sure why this is required
         *
         * these spot instance permissions apply if docker-machine is
         * invoked with --amazonec2-request-spot-instance:
         *
         * "DescribeSpotInstances"
         * "DescribeSpotInstanceRequests"
         * "RequestSpotInstances"
         * "CancelSpotInstanceRequests"
         *
         * "DescribeInstances" -- required to tell when an instance is
         * ready, what its IP address is, etc
         *
         * "RunInstances" -- required to run an AWS instance if not
         * --amazonec2-request-spot-instance
         *
         * "StartInstances" -- docker-machine start
         *
         * "StopInstances" -- docker-machine stop or docker-machine
         * kill
         *
         * "RebootInstances" -- docker-machine restart
         *
         * "TerminateInstances" -- docker-machine rm
         *
         * "CreateTags" -- required to set the Name tag and anything
         * that's passed via --amazonec2-tags
         *
         * "route53:ListHostedZonesByName" -- required to find the
         * hosted zone ID
         *
         * "route53:ChangeResourceRecordSets" -- required to add A
         * records to make DGraph discoverable.
         */
        const statement = new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: [
                "ec2:DescribeSecurityGroups",
                "ec2:CreateSecurityGroup",
                "ec2:DescribeSubnets",
                "ec2:DescribeKeyPairs",
                "ec2:CreateKeyPair",
                "ec2:DescribeSpotInstances",
                "ec2:DescribeSpotInstanceRequests",
                "ec2:RequestSpotInstances",
                "ec2:CancelSpotInstanceRequests",
                "ec2:DescribeInstances",
                "ec2:RunInstances",
                "ec2:StartInstances",
                "ec2:StopInstances",
                "ec2:RebootInstances",
                "ec2:TerminateInstances",
                "ec2:CreateTags",
                "route53:ListHostedZonesByName",
                "route53:ChangeResourceRecordSets"
            ],
            resources: ["*"]
        });

        bastion.role.addToPrincipalPolicy(statement);
    }

    public allowConnectionsFrom(
        other: ec2.IConnectable, portRange: ec2.Port
    ): void {
        this.swarmSecurityGroup.connections.allowFrom(other, portRange);
    }
}
