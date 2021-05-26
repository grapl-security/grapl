import * as cdk from "@aws-cdk/core";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as logs from "@aws-cdk/aws-logs";
import * as path from "path";
import * as s3deploy from "@aws-cdk/aws-s3-deployment";
import { Swarm, SwarmConnectable } from "../swarm";
import { Watchful } from "cdk-watchful";
import { GraplS3Bucket } from "../grapl_s3_bucket";

export interface DGraphSwarmClusterProps {
    deploymentName: string;
    version: string;
    vpc: ec2.IVpc;
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

        const logGroup = new logs.LogGroup(this, "logGroup", {
            logGroupName: `${props.deploymentName.toLowerCase()}-grapl-dgraph`,
            removalPolicy: cdk.RemovalPolicy.DESTROY,
            retention: logs.RetentionDays.ONE_WEEK,
        });

        this.dgraphSwarmCluster = new Swarm(this, "SwarmCluster", {
            deploymentName: props.deploymentName,
            version: props.version,
            vpc: props.vpc,
            logsGroupResourceArn: logGroup.logGroupArn,
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
                ec2.Port.tcp(9083),
            ],
            watchful: props.watchful,
        });

        const dgraphConfigBucket = new GraplS3Bucket(
            this,
            "DGraphConfigBucket",
            {
                bucketName: `${props.deploymentName.toLowerCase()}-dgraph-config-bucket`,
                publicReadAccess: false,
                removalPolicy: cdk.RemovalPolicy.DESTROY,
            }
        );
        // grant read access to the swarm instances
        dgraphConfigBucket.grantRead(this.dgraphSwarmCluster.swarmInstanceRole);

        const dgraphDir = path.join(__dirname, "../../dgraph/");
        new s3deploy.BucketDeployment(this, "dgraphConfigDeployment", {
            sources: [s3deploy.Source.asset(dgraphDir)],
            destinationBucket: dgraphConfigBucket,
        });
    }

    public alphaHost(): string {
        return this.dgraphSwarmCluster.clusterHost();
    }

    public alphaPort(): number {
        return this.dgraphSwarmCluster.clusterPort();
    }

    public alphaHostPort(): string {
        return this.dgraphSwarmCluster.clusterHostPort();
    }

    public allowConnectionsFrom(other: SwarmConnectable): void {
        this.dgraphSwarmCluster.allowConnectionsFrom(other, ec2.Port.tcp(9080));
    }
}
