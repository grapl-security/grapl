import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as iam from '@aws-cdk/aws-iam';

export interface SwarmProps {
    // The VPC where the Docker Swarm cluster will live
    readonly vpc: ec2.IVpc;

    // The service-specific (e.g. DGraph) ports to open internally
    // within the Docker Swarm cluster.
    readonly servicePorts: ec2.Port[];
}

export class Swarm extends cdk.Construct {
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
        const servicePorts = swarmProps.servicePorts;
        servicePorts.forEach(
            (port, _) => swarmSecurityGroup.connections.allowInternally(port)
        );

        new ec2.BastionHostLinux(this, 'bastion', {
            vpc: swarmProps.vpc,
            securityGroup: swarmSecurityGroup,
            instanceType: new ec2.InstanceType("t3.nano"),
            instanceName: "SwarmBastion"
        });
    }
}
