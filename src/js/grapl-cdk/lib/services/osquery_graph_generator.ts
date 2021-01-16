import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3 from '@aws-cdk/aws-s3';
import { Service } from '../service';
import { EventEmitter } from '../event_emitters';
import { RedisCluster } from '../redis';
import { GraplServiceProps } from '../grapl-cdk-stack';

interface OSQueryGraphGeneratorProps extends GraplServiceProps {
    writesTo: s3.IBucket;
}

export class OSQueryGraphGenerator extends cdk.NestedStack {
    constructor(
        parent: cdk.Construct,
        id: string,
        props: OSQueryGraphGeneratorProps
    ) {
        super(parent, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const osquery_log = new EventEmitter(
            this,
            bucket_prefix + '-osquery-log'
        );

        const event_cache = new RedisCluster(this, 'OSQueryEventCache', props);
        event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                RUST_LOG: props.osquerySubgraphGeneratorLogLevel,
                BUCKET_PREFIX: bucket_prefix,
                EVENT_CACHE_CLUSTER_ADDRESS: event_cache.address,
            },
            vpc: props.vpc,
            reads_from: osquery_log.bucket,
            subscribes_to: osquery_log.topic,
            writes_to: props.writesTo,
            version: props.version,
            watchful: props.watchful,
            metric_forwarder: props.metricForwarder,
        });

        service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(parseInt(event_cache.cluster.attrRedisEndpointPort))
        );

        service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(parseInt(event_cache.cluster.attrRedisEndpointPort))
        );
    }
}
