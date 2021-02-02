import * as path from 'path';
import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3 from '@aws-cdk/aws-s3';
import * as sns from '@aws-cdk/aws-sns';
import { Service } from '../service';
import { EventEmitter } from '../event_emitters';
import { RedisCluster } from '../redis';
import { GraplServiceProps } from '../grapl-cdk-stack';
import { GraplS3Bucket } from '../grapl_s3_bucket';
import {FargateService} from "../fargate_service";
import {ContainerImage} from "@aws-cdk/aws-ecs";

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

        const bucket_prefix = props.prefix.toLowerCase();
        const subgraphs_merged = new EventEmitter(
            this,
            bucket_prefix + '-subgraphs-merged'
        );
        const analyzer_bucket = s3.Bucket.fromBucketName(
            this,
            'analyzers-bucket',
            bucket_prefix + "-analyzers-bucket"
        );
        this.bucket = subgraphs_merged.bucket;
        this.topic = subgraphs_merged.topic;

        const dispatch_event_cache = new RedisCluster(
            this,
            'DispatchedEventCache',
            props
        );
        dispatch_event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        this.service = new FargateService(this, id, {
            prefix: props.prefix,
            environment: {
                RUST_LOG: props.analyzerDispatcherLogLevel,
                ANALYZER_BUCKET: bucket_prefix + "-analyzers-bucket",
                EVENT_CACHE_CLUSTER_ADDRESS: dispatch_event_cache.address,
                DISPATCHED_ANALYZER_BUCKET: props.writesTo.bucketName,
                SUBGRAPH_MERGED_BUCKET: subgraphs_merged.bucket.bucketName,
            },
            vpc: props.vpc,
            eventEmitter: subgraphs_merged,
            writesTo: props.writesTo,
            version: props.version,
            watchful: props.watchful,
            serviceImage: ContainerImage.fromAsset(path.join(__dirname, '../../../../../src/rust/'), {
                target: "analyzer-dispatcher-deploy",
                buildArgs: {
                    "CARGO_PROFILE": "debug"
                },
                file: "Dockerfile",
            }),
            command: ["/analyzer-dispatcher"],
            // metric_forwarder: props.metricForwarder,
        });
        analyzer_bucket.grantRead(this.service.service.service.taskDefinition.taskRole);
        analyzer_bucket.grantRead(this.service.retryService.service.taskDefinition.taskRole);
        this.service.service.cluster.connections.allowToAnyIpv4(
            ec2.Port.tcp(parseInt(dispatch_event_cache.cluster.attrRedisEndpointPort))
        );

    }
}
