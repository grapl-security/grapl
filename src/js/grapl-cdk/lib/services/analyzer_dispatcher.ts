import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3 from '@aws-cdk/aws-s3';
import * as sns from '@aws-cdk/aws-sns';
import { Service } from '../service';
import { EventEmitter } from '../event_emitters';
import { RedisCluster } from '../redis';
import { GraplServiceProps } from '../grapl-cdk-stack';
import { GraplS3Bucket } from '../grapl_s3_bucket';

export interface AnalyzerDispatchProps extends GraplServiceProps {
    writesTo: s3.IBucket;
    readsFrom: s3.IBucket;
}

export class AnalyzerDispatch extends cdk.NestedStack {
    readonly bucket: GraplS3Bucket;
    readonly topic: sns.Topic;
    readonly service: Service;

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
        this.bucket = subgraphs_merged.bucket;
        this.topic = subgraphs_merged.topic;

        const dispatch_event_cache = new RedisCluster(
            this,
            'DispatchedEventCache',
            props
        );
        dispatch_event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        this.service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                BUCKET_PREFIX: bucket_prefix,
                EVENT_CACHE_ADDR: dispatch_event_cache.cluster.attrRedisEndpointAddress,
                EVENT_CACHE_PORT: dispatch_event_cache.cluster.attrRedisEndpointPort,
                DISPATCHED_ANALYZER_BUCKET: props.writesTo.bucketName,
                SUBGRAPH_MERGED_BUCKET: subgraphs_merged.bucket.bucketName,
            },
            vpc: props.vpc,
            reads_from: subgraphs_merged.bucket,
            subscribes_to: subgraphs_merged.topic,
            writes_to: props.writesTo,
            version: props.version,
            watchful: props.watchful,
            metric_forwarder: props.metricForwarder,
        });

        this.service.readsFrom(props.readsFrom, true);

        this.service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.allTcp(),
            'Allow outbound to S3'
        );
        this.service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.allTcp(),
            'Allow outbound to S3'
        );
    }
}
