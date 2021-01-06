import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3 from '@aws-cdk/aws-s3';
import { Service } from '../service';
import { EventEmitter } from '../event_emitters';
import { RedisCluster } from '../redis';
import { GraplServiceProps } from '../grapl-cdk-stack';

interface SysmonGraphGeneratorProps extends GraplServiceProps {
    writesTo: s3.IBucket;
}

export class SysmonGraphGenerator extends cdk.NestedStack {
    readonly service: Service;

    constructor(
        parent: cdk.Construct,
        id: string,
        props: SysmonGraphGeneratorProps
    ) {
        super(parent, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const sysmon_log = new EventEmitter(
            this,
            bucket_prefix + '-sysmon-log'
        );

        const event_cache = new RedisCluster(this, 'SysmonEventCache', props);
        event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        this.service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                BUCKET_PREFIX: bucket_prefix,
                EVENT_CACHE_ADDR: event_cache.cluster.attrRedisEndpointAddress,
                EVENT_CACHE_PORT: event_cache.cluster.attrRedisEndpointPort,
            },
            vpc: props.vpc,
            reads_from: sysmon_log.bucket,
            subscribes_to: sysmon_log.topic,
            writes_to: props.writesTo,
            version: props.version,
            watchful: props.watchful,
            metric_forwarder: props.metricForwarder,
        });

        this.service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(parseInt(event_cache.cluster.attrRedisEndpointPort))
        );

        this.service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(parseInt(event_cache.cluster.attrRedisEndpointPort))
        );
    }
}
