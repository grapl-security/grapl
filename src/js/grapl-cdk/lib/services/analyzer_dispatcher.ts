import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3 from '@aws-cdk/aws-s3';
import * as sns from '@aws-cdk/aws-sns';
import { EventEmitter } from '../event_emitters';
import { RedisCluster } from '../redis';
import { GraplServiceProps } from '../grapl-cdk-stack';
import { GraplS3Bucket } from '../grapl_s3_bucket';
import {FargateService} from "../fargate_service";
import {ContainerImage} from "@aws-cdk/aws-ecs";
import { SRC_DIR, RUST_DOCKERFILE } from '../dockerfile_paths';

export interface AnalyzerDispatchProps extends GraplServiceProps {
    writesTo: s3.IBucket;
    readsFrom: s3.IBucket;
}

export class AnalyzerDispatch extends cdk.NestedStack {
    readonly bucket: GraplS3Bucket;
    readonly topic: sns.Topic;
    readonly service: FargateService;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: AnalyzerDispatchProps
    ) {
        super(scope, id);

        const service_name = "analyzer-dispatcher";
        const deployment_name = props.deploymentName.toLowerCase();
        const subgraphs_merged = new EventEmitter(
            this,
            deployment_name + '-subgraphs-merged'
        );
        const analyzer_bucket = s3.Bucket.fromBucketName(
            this,
            'analyzers-bucket',
            deployment_name + "-analyzers-bucket"
        );
        this.bucket = subgraphs_merged.bucket;
        this.topic = subgraphs_merged.topic;

        const dispatch_event_cache = new RedisCluster(
            this,
            'DispatchedEventCache',
            props
        );
        dispatch_event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        this.service = new FargateService(this, service_name, {
            deploymentName: props.deploymentName,
            environment: {
                ANALYZER_BUCKET: deployment_name + "-analyzers-bucket",
                RUST_LOG: props.logLevels.analyzerDispatcherLogLevel,
                REDIS_ENDPOINT: dispatch_event_cache.address,
                DISPATCHED_ANALYZER_BUCKET: props.writesTo.bucketName,
                SUBGRAPH_MERGED_BUCKET: subgraphs_merged.bucket.bucketName,
            },
            vpc: props.vpc,
            eventEmitter: subgraphs_merged,
            writesTo: props.writesTo,
            version: props.version,
            watchful: props.watchful,
            serviceImage: ContainerImage.fromAsset(SRC_DIR, {
                target: "analyzer-dispatcher-deploy",
                buildArgs: {
                    "CARGO_PROFILE": "debug"
                },
                file: RUST_DOCKERFILE,
            }),
            command: ["/analyzer-dispatcher"],
            metric_forwarder: props.metricForwarder,
        });
        analyzer_bucket.grantRead(this.service.service.service.taskDefinition.taskRole);
        analyzer_bucket.grantRead(this.service.retryService.service.taskDefinition.taskRole);
        this.service.service.cluster.connections.allowToAnyIpv4(
            ec2.Port.tcp(parseInt(dispatch_event_cache.cluster.attrRedisEndpointPort))
        );
    }
}
