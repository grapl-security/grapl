import * as asg from '@aws-cdk/aws-autoscaling'
import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as iam from '@aws-cdk/aws-iam';
import * as lambda from '@aws-cdk/aws-lambda';
import * as route53 from '@aws-cdk/aws-route53';
import * as s3 from '@aws-cdk/aws-s3';
import * as s3deploy from '@aws-cdk/aws-s3-deployment';

import * as path from 'path';

import { FunctionHook } from '@aws-cdk/aws-autoscaling-hooktargets';
import { Watchful } from 'cdk-watchful';
import { Duration } from '@aws-cdk/core';

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

    // CDK Watchful instance for monitoring the lifecycle event
    // listener lambda.
    readonly watchful?: Watchful;
}

export class Swarm extends cdk.Construct {
    private readonly swarmHostedZone: route53.PrivateHostedZone;
    private readonly swarmAsg: asg.AutoScalingGroup;

    constructor(scope: cdk.Construct, id: string, props: SwarmProps) {
        super(scope, id);

        const swarmSecurityGroup = new ec2.SecurityGroup(scope, 'Swarm', {
            description: `${props.prefix} DGraph Swarm security group`,
            vpc: props.vpc,
            allowAllOutbound: false,
        });

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
                vpc: props.vpc,
                zoneName: props.prefix.toLowerCase() + '.dgraph.grapl',
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
                'ec2:DescribeAddresses',
            ],
            resources: [
                '*', // FIXME: lock this down more?
            ]
        }));
        lifecycleListenerRole.addToPrincipalPolicy(new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: [
                'route53:ListResourceRecordSets',
                'route53:ChangeResourceRecordSets',
            ],
            resources: [
                `arn:aws:route53:::hostedzone/${this.swarmHostedZone.hostedZoneId}`,
            ]
        }));

        // Swarm instance lifecycle event listeners
        const launchListener = new lambda.Function(this, "SwarmLaunchListener", {
            code: lambda.Code.fromAsset(
                `./zips/swarm-lifecycle-event-handler-${props.version}.zip`
            ),
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: 'app.main',
            role: lifecycleListenerRole,
            environment: {
                GRAPL_LOG_LEVEL: 'INFO',
            },
            timeout: Duration.seconds(300),
            reservedConcurrentExecutions: 1,
        });
        const terminateListener = new lambda.Function(this, "SwarmTerminateListener", {
            code: lambda.Code.fromAsset(
                `./zips/swarm-lifecycle-event-handler-${props.version}.zip`
            ),
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: 'app.main',
            role: lifecycleListenerRole,
            environment: {
                GRAPL_LOG_LEVEL: 'INFO',
            },
            timeout: Duration.seconds(300),
            reservedConcurrentExecutions: 1,
        });

        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                launchListener.functionName,
                launchListener
            );
            props.watchful.watchLambdaFunction(
                terminateListener.functionName,
                terminateListener
            );
        }

        // Swarm cluster ASG
        const zoneName = props.prefix.toLowerCase() + '.dgraph.grapl';
        const swarmAsg = new asg.AutoScalingGroup(this, 'SwarmASG', {
            vpc: props.vpc,
            instanceType: props.instanceType,
            userData: swarmUserData,
            machineImage: ec2.MachineImage.genericLinux(amazonLinux2Amis),
            role: swarmInstanceRole,
            desiredCapacity: 0,
            minCapacity: 0,
            maxCapacity: 0,
        });

        lifecycleListenerRole.addToPrincipalPolicy(new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: [
                'autoscaling:CompleteLifecycleAction',
            ],
            resources: [
                swarmAsg.autoScalingGroupArn,
            ]
        }));

        const metadata = `{"HostedZoneId":"${this.swarmHostedZone.hostedZoneId}","DnsName":"${zoneName}","Prefix":"${props.prefix}","AsgName":"${swarmAsg.autoScalingGroupName}"}`;

        swarmAsg.addSecurityGroup(swarmSecurityGroup);
        swarmAsg.addLifecycleHook('SwarmLaunchHook', {
            lifecycleTransition: asg.LifecycleTransition.INSTANCE_LAUNCHING,
            notificationTarget: new FunctionHook(launchListener),
            defaultResult: asg.DefaultResult.ABANDON,
            notificationMetadata: metadata,
            lifecycleHookName: `${props.prefix}-SwarmLaunchHook`,
        });
        swarmAsg.addLifecycleHook('SwarmTerminateHook', {
            lifecycleTransition: asg.LifecycleTransition.INSTANCE_TERMINATING,
            notificationTarget: new FunctionHook(terminateListener),
            defaultResult: asg.DefaultResult.CONTINUE,
            notificationMetadata: metadata,
            lifecycleHookName: `${props.prefix}-SwarmTerminateHook`,
        });

        this.swarmAsg = swarmAsg;

        // Deploy cluster setup scripts to S3
        const swarmConfigBucket = new s3.Bucket(this, 'SwarmConfigBucket', {
            bucketName: `${props.prefix.toLowerCase()}-swarm-config-bucket`,
            publicReadAccess: false,
            removalPolicy: cdk.RemovalPolicy.DESTROY,
        });
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
        this.swarmAsg.connections.allowFrom(other, portRange);
    }

    public clusterHostPort(): string {
        return `http://${this.swarmHostedZone.zoneName}:9080`;
    }
}
