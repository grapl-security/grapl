import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3deploy from '@aws-cdk/aws-s3-deployment';
import * as path from 'path';
import { Swarm } from '../swarm';
import { Watchful } from 'cdk-watchful';
import { GraplS3Bucket } from '../grapl_s3_bucket';

export interface DGraphSwarmClusterProps {
    prefix: string;
    version: string;
    vpc: ec2.IVpc;
    instanceType: ec2.InstanceType;
    watchful?: Watchful;
}

export class DGraphSwarmCluster extends cdk.NestedStack {
    private readonly dgraphSwarmCluster: Swarm;

    constructor(
        parent: cdk.Construct,
        id: string,
        props: DGraphSwarmClusterProps
    ) {
        super(parent, id);

        this.dgraphSwarmCluster = new Swarm(this, 'SwarmCluster', {
            prefix: props.prefix,
            version: props.version,
            vpc: props.vpc,
            logsGroupResourceArn: super.formatArn({
                partition: 'aws',
                service: 'logs',
                resource: 'log-group',
                sep: ':',
                resourceName: `${props.prefix.toLowerCase()}-grapl-dgraph`
            }),
            internalServicePorts: [
                ec2.Port.tcp(5080),
                ec2.Port.tcp(6080),
                ec2.Port.tcp(7081),
                ec2.Port.tcp(7082),
                ec2.Port.tcp(7083),
                ec2.Port.tcp(8081),
                ec2.Port.tcp(8082),
                ec2.Port.tcp(8083),
                ec2.Port.tcp(9081),
                ec2.Port.tcp(9082),
                ec2.Port.tcp(9083)
            ],
            instanceType: props.instanceType,
            watchful: props.watchful,
        });

        const dgraphConfigBucket = new GraplS3Bucket(this, 'DGraphConfigBucket', {
            bucketName: `${props.prefix.toLowerCase()}-dgraph-config-bucket`,
            publicReadAccess: false,
            removalPolicy: cdk.RemovalPolicy.DESTROY,
        });
        // grant read access to the swarm instances
        dgraphConfigBucket.grantRead(this.dgraphSwarmCluster.swarmInstanceRole);

        const dgraphDir = path.join(__dirname, '../../dgraph/');
        new s3deploy.BucketDeployment(this, "dgraphConfigDeployment", {
            sources: [s3deploy.Source.asset(dgraphDir)],
            destinationBucket: dgraphConfigBucket,
        });
    }

    public alphaHostPort(): string {
        return this.dgraphSwarmCluster.clusterHostPort();
    }

    public allowConnectionsFrom(other: ec2.IConnectable): void {
        this.dgraphSwarmCluster.allowConnectionsFrom(other, ec2.Port.tcp(9080));
    }
}
