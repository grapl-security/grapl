import * as asg from '@aws-cdk/aws-autoscaling'
import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as iam from '@aws-cdk/aws-iam';
import * as lambda from '@aws-cdk/aws-lambda';
import * as route53 from '@aws-cdk/aws-route53';

import { FunctionHook } from '@aws-cdk/aws-autoscaling-hooktargets';

import { Watchful } from './vendor/cdk-watchful/lib/watchful';

export interface SwarmProps {
    // Grapl deployment name prefix.
    readonly prefix: String;

    // Grapl deployment version.
    readonly version: String;

    // The VPC where the Docker Swarm cluster will run.
    readonly vpc: ec2.IVpc;

    // The service-specific (e.g. DGraph) ports to open internally
    // within the Docker Swarm cluster.
    readonly internalServicePorts: ec2.Port[];

    // The EC2 Instance Type for the Docker Swarm instances.
    readonly instanceType: ec2.InstanceType;

    // Number of Docker Swarm instances in the cluster.
    readonly clusterSize: number;

    // CDK Watchful instance for monitoring the lifecycle event
    // listener lambda.
    readonly watchful?: Watchful;
}

export class Swarm extends cdk.Construct {
    private readonly swarmHostedZone: route53.PrivateHostedZone;
    private readonly swarmAsg: asg.AutoScalingGroup;

    constructor(scope: cdk.Construct, id: string, swarmProps: SwarmProps) {
        super(scope, id);

        const swarmSecurityGroup = new ec2.SecurityGroup(scope, 'Swarm', {
            description: `${swarmProps.prefix} DGraph Swarm security group`,
            vpc: swarmProps.vpc,
            allowAllOutbound: false,
        });

        // allow the bastion machine to make outbound connections to
        // the Internet for these services:
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
        swarmProps.internalServicePorts.forEach((port, _) =>
            swarmSecurityGroup.connections.allowInternally(port)
        );

        // Spin up a bastion instance. This is where users will
        // perform cluster maintenance tasks.
        const bastion = new ec2.BastionHostLinux(this, 'Bastion', {
            vpc: swarmProps.vpc,
            securityGroup: swarmSecurityGroup,
            instanceType: new ec2.InstanceType('t3a.nano'),
            instanceName: swarmProps.prefix + '-SwarmBastion',
        });

        // UserData commands for initializing bastion instance.
        [
            'yum install -y docker',
            'systemctl enable docker.service',
            'systemctl start docker.service',
            'usermod -a -G docker ssm-user',
        ].forEach((cmd, _) => bastion.instance.addUserData(cmd));

        // IAM Role for Swarm instances
        const swarmInstanceRole = new iam.Role(this, 'SwarmRole', {
            assumedBy: new iam.ServicePrincipal('ec2.amazonaws.com')
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
                'arn:aws:logs:*:*:*', // FIXME: lock this down more?
            ],
        }));

        // UserData commands for initializing the Swarm instances.
        const swarmUserData = ec2.UserData.forLinux();
        swarmUserData.addCommands(...[
            'yum install -y docker amazon-cloudwatch-agent',
            'amazon-cloudwatch-agent-ctl -m ec2 -a start',
            'systemctl enable docker.service',
            'systemctl start docker.service',
            'usermod -a -G docker ec2-user',
            'mkdir /dgraph',
            'mkfs -t xfs /dev/nvme0n1',
            'mount -t xfs /dev/nvme0n1 /dgraph',
            'UUID=$(lsblk -o +UUID | grep nvme0n1 | rev | cut -d" " -f1 | rev); echo -e "UUID=$UUID\t/dgraph\txfs\tdefaults,nofail\t0 2" | tee -a /etc/fstab'
        ]);

        // Configure a Route53 Hosted Zone for the Swarm cluster.
        this.swarmHostedZone = new route53.PrivateHostedZone(
            this,
            'SwarmZone',
            {
                vpc: swarmProps.vpc,
                zoneName: swarmProps.prefix.toLowerCase() + '.dgraph.grapl',
            }
        );

        // This mapping was compiled on 2020-10-14 by running the
        // following query for each region:
        //
        // aws ec2 describe-images \
        //  --owners amazon \
        //  --filters 'Name=name,Values=amzn2-ami-hvm-2.0.????????.?-x86_64-gp2' 'Name=state,Values=available' \
        //  --query 'reverse(sort_by(Images, &CreationDate))[:1]' \
        //  --region us-east-1
        //
        // It should probably be updated periodically. Be careful that
        // if you change one of these AMI IDs you don't accidentally
        // blow away Docker Swarm EC2 instances.
        const amazonLinux2Amis = {
            'us-east-1': 'ami-0947d2ba12ee1ff75',
            'us-east-2': 'ami-03657b56516ab7912',
            'us-west-1': 'ami-0e4035ae3f70c400f',
            'us-west-2': 'ami-0528a5175983e7f28'
        }

        // IAM role for lifecycle event listener
        const lifecycleListenerRole = new iam.Role(this, 'LifecycleListenerRole', {
            assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
            managedPolicies: [
                iam.ManagedPolicy.fromAwsManagedPolicyName(
                    'service-role/AWSLambdaBasicExecutionRole' // FIXME: remove managed policy
                ),
                iam.ManagedPolicy.fromAwsManagedPolicyName(
                    'service-role/AWSLambdaVPCAccessExecutionRole' // FIXME: remove managed policy
                ),
            ],
        });
        lifecycleListenerRole.addToPrincipalPolicy(new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: [
                'ec2:DescribeInstances',
            ],
            resources: [
                'arn:aws:ec2:::instance/*', // FIXME: lock this down more?
            ]
        }));
        lifecycleListenerRole.addToPrincipalPolicy(new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: [
                'route53:ChangeResourceRecordSets',
            ],
            resources: [
                `arn:aws:route53:::hostedzone/${this.swarmHostedZone.hostedZoneId}`,
            ]
        }));

        // Swarm instance lifecycle event listeners
        const launchListener = new lambda.Function(this, "SwarmLaunchListener", {
            code: lambda.Code.fromAsset(
                `./zips/swarm-lifecycle-event-handler-${swarmProps.version}.zip`
            ),
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: 'app.main',
            role: lifecycleListenerRole,
        });
        const terminateListener = new lambda.Function(this, "SwarmTerminateListener", {
            code: lambda.Code.fromAsset(
                `./zips/swarm-lifecycle-event-handler-${swarmProps.version}.zip`
            ),
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: 'app.main',
            role: lifecycleListenerRole,
        });

        if (swarmProps.watchful) {
            swarmProps.watchful.watchLambdaFunction(
                launchListener.functionName,
                launchListener
            );
            swarmProps.watchful.watchLambdaFunction(
                terminateListener.functionName,
                terminateListener
            );
        }

        // Swarm cluster ASG
        const zoneName = swarmProps.prefix.toLowerCase() + '.dgraph.grapl';
        const metadata = `{"HostedZoneId":${this.swarmHostedZone.hostedZoneId},"DnsName":${zoneName},"Prefix":${swarmProps.prefix}}`;
        const swarmAsg = new asg.AutoScalingGroup(this, 'SwarmASG', {
            vpc: swarmProps.vpc,
            instanceType: swarmProps.instanceType,
            userData: swarmUserData,
            machineImage: ec2.MachineImage.genericLinux(amazonLinux2Amis),
            role: swarmInstanceRole,
            desiredCapacity: swarmProps.clusterSize,
            minCapacity: swarmProps.clusterSize,
            maxCapacity: swarmProps.clusterSize,
        });
        swarmAsg.addSecurityGroup(swarmSecurityGroup);
        swarmAsg.addLifecycleHook('SwarmLaunchHook', {
            lifecycleTransition: asg.LifecycleTransition.INSTANCE_LAUNCHING,
            notificationTarget: new FunctionHook(launchListener),
            defaultResult: asg.DefaultResult.ABANDON,
            notificationMetadata: metadata,
            lifecycleHookName: `${swarmProps.prefix}-SwarmLaunchHook`,
        });
        swarmAsg.addLifecycleHook('SwarmTerminateHook', {
            lifecycleTransition: asg.LifecycleTransition.INSTANCE_TERMINATING,
            notificationTarget: new FunctionHook(terminateListener),
            defaultResult: asg.DefaultResult.CONTINUE,
            notificationMetadata: metadata,
            lifecycleHookName: `${swarmProps.prefix}-SwarmTerminateHook`,
        });

        this.swarmAsg = swarmAsg;
    }

    public allowConnectionsFrom(
        other: ec2.IConnectable,
        portRange: ec2.Port
    ): void {
        this.swarmAsg.connections.allowFrom(other, portRange);
    }

    public clusterHostPort(): string {
        return `http://${this.swarmHostedZone.zoneName}:9080`;
    }
}
