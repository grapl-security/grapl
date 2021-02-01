import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as iam from '@aws-cdk/aws-iam';
import * as lambda from '@aws-cdk/aws-lambda';
import * as s3 from '@aws-cdk/aws-s3';
import { Service } from '../service';
import { EventEmitter } from '../event_emitters';
import { RedisCluster } from '../redis';
import { GraplServiceProps } from '../grapl-cdk-stack';

export interface AnalyzerExecutorProps extends GraplServiceProps {
    writesTo: s3.IBucket;
    readsAnalyzersFrom: s3.IBucket;
    modelPluginsBucket: s3.IBucket;
}

export class AnalyzerExecutor extends cdk.NestedStack {
    readonly bucket: s3.IBucket;
    readonly service: Service;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: AnalyzerExecutorProps
    ) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const dispatched_analyzer = new EventEmitter(
            this,
            bucket_prefix + '-dispatched-analyzer'
        );
        this.bucket = dispatched_analyzer.bucket;

        const count_cache = new RedisCluster(this, 'ExecutorCountCache', props);
        const hit_cache = new RedisCluster(this, 'ExecutorHitCache', props);
        const message_cache = new RedisCluster(this, 'ExecutorMsgCache', props);

        this.service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                ANALYZER_MATCH_BUCKET: props.writesTo.bucketName,
                BUCKET_PREFIX: bucket_prefix,
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                COUNTCACHE_ADDR: count_cache.cluster.attrRedisEndpointAddress,
                COUNTCACHE_PORT: count_cache.cluster.attrRedisEndpointPort,
                MESSAGECACHE_ADDR: message_cache.cluster.attrRedisEndpointAddress,
                MESSAGECACHE_PORT: message_cache.cluster.attrRedisEndpointPort,
                HITCACHE_ADDR: hit_cache.cluster.attrRedisEndpointAddress,
                HITCACHE_PORT: hit_cache.cluster.attrRedisEndpointPort,
                GRAPL_LOG_LEVEL: 'INFO',
                GRPC_ENABLE_FORK_SUPPORT: '1',
            },
            vpc: props.vpc,
            reads_from: dispatched_analyzer.bucket,
            writes_to: props.writesTo,
            subscribes_to: dispatched_analyzer.topic,
            opt: {
                runtime: lambda.Runtime.PYTHON_3_7,
                py_entrypoint: "lambda_function.lambda_handler"
            },
            version: props.version,
            watchful: props.watchful,
            metric_forwarder: props.metricForwarder,
        }
        );
        const service = this.service;

        props.dgraphSwarmCluster.allowConnectionsFrom(service.event_handler);

        // We need the List capability to find each of the analyzers
        props.readsAnalyzersFrom.grantRead(service.event_handler);
        props.readsAnalyzersFrom.grantRead(service.event_retry_handler);

        service.readsFrom(props.modelPluginsBucket, true);

        // Need to be able to GetObject in order to HEAD, can be replaced with
        // a cache later, but safe so long as there is no LIST
        const policy = new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: ['s3:GetObject'],
            resources: [props.writesTo.bucketArn + '/*'],
        });

        service.event_handler.addToRolePolicy(policy);
        service.event_retry_handler.addToRolePolicy(policy);

        service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.allTraffic(),
            'Allow outbound to S3'
        );
        service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.allTraffic(),
            'Allow outbound to S3'
        );
    }
}
