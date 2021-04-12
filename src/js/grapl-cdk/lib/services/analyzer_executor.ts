import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as iam from '@aws-cdk/aws-iam';
import * as s3 from '@aws-cdk/aws-s3';
import { ContainerImage } from "@aws-cdk/aws-ecs";
import { EventEmitter } from '../event_emitters';
import { FargateService } from '../fargate_service';
import { GraplServiceProps } from '../grapl-cdk-stack';
import { SRC_DIR, PYTHON_DOCKERFILE } from '../dockerfile_paths';
import { RedisCluster } from '../redis';

export interface AnalyzerExecutorProps extends GraplServiceProps {
    writesTo: s3.IBucket;
    readsAnalyzersFrom: s3.IBucket;
    modelPluginsBucket: s3.IBucket;
}

export class AnalyzerExecutor extends cdk.NestedStack {
    readonly sourceBucket: s3.IBucket;
    readonly service: FargateService;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: AnalyzerExecutorProps
    ) {
        super(scope, id);

        const deployment_name = props.deploymentName.toLowerCase();
        const dispatched_analyzer = new EventEmitter(
            this,
            deployment_name + '-dispatched-analyzer'
        );
        this.sourceBucket = dispatched_analyzer.bucket;

        const count_cache = new RedisCluster(this, 'ExecutorCountCache', props);
        const hit_cache = new RedisCluster(this, 'ExecutorHitCache', props);
        const message_cache = new RedisCluster(this, 'ExecutorMsgCache', props);

        this.service = new FargateService(this, id, {
                deploymentName: props.deploymentName,
                environment: {
                    ANALYZER_MATCH_BUCKET: props.writesTo.bucketName,
                    DEPLOYMENT_NAME: deployment_name,
                    MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                    COUNTCACHE_ADDR: count_cache.cluster.attrRedisEndpointAddress,
                    COUNTCACHE_PORT: count_cache.cluster.attrRedisEndpointPort,
                    MESSAGECACHE_ADDR:
                      message_cache.cluster.attrRedisEndpointAddress,
                    MESSAGECACHE_PORT: message_cache.cluster.attrRedisEndpointPort,
                    HITCACHE_ADDR: hit_cache.cluster.attrRedisEndpointAddress,
                    HITCACHE_PORT: hit_cache.cluster.attrRedisEndpointPort,
                    GRAPL_LOG_LEVEL: props.logLevels.analyzerExecutorLogLevel,
                    GRPC_ENABLE_FORK_SUPPORT: '1',
                },
                vpc: props.vpc,
                eventEmitter: dispatched_analyzer,
                writesTo: props.writesTo,
                version: props.version,
                watchful: props.watchful,
                serviceImage: ContainerImage.fromAsset(SRC_DIR, {
                    target: "analyzer-executor",
                    file: PYTHON_DOCKERFILE,
                }),
                metric_forwarder: props.metricForwarder,
            },
        );
        const service = this.service;

        props.dgraphSwarmCluster.allowConnectionsFrom(service.service.service);
        props.dgraphSwarmCluster.allowConnectionsFrom(service.retryService.service);

        // We need the List capability to find each of the analyzers
        service.readsFromBucket(props.readsAnalyzersFrom, true)
        service.readsFromBucket(props.modelPluginsBucket, true);

        // Need to be able to GetObject in order to HEAD, can be replaced with
        // a cache later, but safe so long as there is no LIST
        service.readsFromBucket(props.writesTo, false);

        service.grantListQueues();

        for (const s of [service.service, service.retryService]) {
            const conn = s.service.connections;
            conn.allowToAnyIpv4(
                ec2.Port.allTraffic(),
                'Allow outbound to S3'
            );
        }
    }
}
