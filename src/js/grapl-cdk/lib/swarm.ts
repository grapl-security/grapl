import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as iam from '@aws-cdk/aws-iam';
import * as route53 from '@aws-cdk/aws-route53';
import * as s3deploy from '@aws-cdk/aws-s3-deployment';

import * as path from 'path';

import { Watchful } from 'cdk-watchful';
import { Tags } from '@aws-cdk/core';
import { GraplS3Bucket } from './grapl_s3_bucket';

export interface SwarmProps {
    // Grapl deployment name prefix.
    readonly prefix: String;

    // Grapl deployment version.
    readonly version: String;

    // The VPC where the Docker Swarm cluster will run.
    readonly vpc: ec2.IVpc;

    // ARN specifying the Grapl logs group for this grapl deployment
    //
    // Should have the following structure:
    // arn:aws:logs:{region}:{account-id}:log-group:{log_group_name}
    readonly logsGroupResourceArn: String;

    // The service-specific (e.g. DGraph) ports to open internally
    // within the Docker Swarm cluster.
    readonly internalServicePorts: ec2.Port[];

    // CDK Watchful instance for monitoring the lifecycle event
    // listener lambda.
    readonly watchful?: Watchful;
}

export class Swarm extends cdk.Construct {
    private readonly swarmHostedZone: route53.PrivateHostedZone;
    private readonly swarmSecurityGroup: ec2.SecurityGroup;
    readonly swarmInstanceRole: iam.Role;

    constructor(scope: cdk.Construct, id: string, props: SwarmProps) {
        super(scope, id);

        const swarmSecurityGroup = new ec2.SecurityGroup(scope, 'Swarm', {
            description: `${props.prefix} Docker Swarm security group`,
            vpc: props.vpc,
            allowAllOutbound: false,
            securityGroupName: `${props.prefix.toLowerCase()}-grapl-swarm`,
        });
        Tags.of(swarmSecurityGroup).add("grapl-deployment-prefix", `${props.prefix.toLowerCase()}`);

        // allow hosts in the swarm security group to make outbound
        // connections to the Internet for these services:
        //   TCP 443 -- AWS SSM Agent (for handshake)
        //   TCP 80 -- yum package manager and wget (install Docker)
        swarmSecurityGroup.connections.allowToAnyIpv4(ec2.Port.tcp(443));
        swarmSecurityGroup.connections.allowToAnyIpv4(ec2.Port.tcp(80));

        // allow hosts in the swarm security group to communicate
        // internally on the following ports:
        //   TCP 2376 -- secure docker client
        //   TCP 2377 -- inter-node communication (only needed on manager nodes)
        //   TCP + UDP 7946 -- container network discovery
        //   UDP 4789 -- overlay network traffic
        swarmSecurityGroup.connections.allowInternally(ec2.Port.tcp(2376));
        swarmSecurityGroup.connections.allowInternally(ec2.Port.tcp(2377));
        swarmSecurityGroup.connections.allowInternally(ec2.Port.tcp(7946));
        swarmSecurityGroup.connections.allowInternally(ec2.Port.udp(7946));
        swarmSecurityGroup.connections.allowInternally(ec2.Port.udp(4789));

        // allow hosts in the swarm security group to communicate
        // internally on the given service ports.
        props.internalServicePorts.forEach((port, _) =>
            swarmSecurityGroup.connections.allowInternally(port)
        );

        this.swarmSecurityGroup = swarmSecurityGroup;

        // IAM Role for Swarm instances
        const swarmInstanceRole = new iam.Role(this, 'SwarmRole', {
            assumedBy: new iam.ServicePrincipal('ec2.amazonaws.com'),
            roleName: `${props.prefix.toLowerCase()}-grapl-swarm-role`,
        });

        // CloudWatchAgentServerPolicy allows the Swarm instances to
        // run the CloudWatch Agent.
        swarmInstanceRole.addManagedPolicy(
            iam.ManagedPolicy.fromAwsManagedPolicyName(
                'CloudWatchAgentServerPolicy' // FIXME: don't use managed policy
            )
        );

        // AmazonSSMManagedInstanceCore allows users to connect to
        // instances with SSM
        swarmInstanceRole.addManagedPolicy(
            iam.ManagedPolicy.fromAwsManagedPolicyName(
                'AmazonSSMManagedInstanceCore' // FIXME: don't use managed policy
            )
        );

        // Logging policy to allow Swarm instances to ship service
        // logs to CloudWatch.
        swarmInstanceRole.addToPrincipalPolicy(new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: [
                'logs:CreateLogGroup',
                'logs:CreateLogStream',
                'logs:PutLogEvents',
                'logs:DescribeLogStreams',
            ],
            resources: [
                `${props.logsGroupResourceArn}:*`
            ],
        }));

        // Configure a Route53 Hosted Zone for the Swarm cluster.
        this.swarmHostedZone = new route53.PrivateHostedZone(
            this,
            'SwarmZone',
            {
                vpc: props.vpc,
                zoneName: `${props.prefix.toLowerCase()}.dgraph.grapl`,
            }
        );

        // Bucket for swarm configs
        const swarmConfigBucket = new GraplS3Bucket(this, 'SwarmConfigBucket', {
            bucketName: `${props.prefix.toLowerCase()}-swarm-config-bucket`,
            publicReadAccess: false,
            removalPolicy: cdk.RemovalPolicy.DESTROY,
        });
        // grant read access to the swarm instances
        swarmConfigBucket.grantRead(swarmInstanceRole);

        this.swarmInstanceRole = swarmInstanceRole;

        // Deploy cluster setup scripts to S3
        const swarmDir = path.join(__dirname, '../swarm/');
        new s3deploy.BucketDeployment(this, 'SwarmConfigDeployment', {
            sources: [s3deploy.Source.asset(swarmDir)],
            destinationBucket: swarmConfigBucket,
        });
    }

    public allowConnectionsFrom(
        other: ec2.IConnectable,
        portRange: ec2.Port
    ): void {
        this.swarmSecurityGroup.connections.allowFrom(other, portRange);
    }

    public clusterHostPort(): string {
        return `${this.swarmHostedZone.zoneName}:9080`;
    }
}
