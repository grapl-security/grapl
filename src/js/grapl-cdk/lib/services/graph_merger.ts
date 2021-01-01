import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3 from '@aws-cdk/aws-s3';
import { Service } from '../service';
import { EventEmitter } from '../event_emitters';
import { RedisCluster } from '../redis';
import { GraplServiceProps } from '../grapl-cdk-stack';
import { SchemaDb } from '../schemadb';

export interface GraphMergerProps extends GraplServiceProps {
    writesTo: s3.IBucket;
    schemaTable: SchemaDb;
}

export class GraphMerger extends cdk.NestedStack {
    readonly bucket: s3.Bucket;
    readonly service: Service;

    constructor(scope: cdk.Construct, id: string, props: GraphMergerProps) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const subgraphs_generated = new EventEmitter(
            this,
            bucket_prefix + '-subgraphs-generated'
        );
        this.bucket = subgraphs_generated.bucket;

        const graph_merge_cache = new RedisCluster(
            this,
            'GraphMergerMergedCache',
            props
        );
        graph_merge_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        this.service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                BUCKET_PREFIX: bucket_prefix,
                SUBGRAPH_MERGED_BUCKET: props.writesTo.bucketName,
                MG_ALPHAS: 'http://' + props.dgraphSwarmCluster.alphaHostPort(),
                MERGED_CACHE_ADDR: graph_merge_cache.cluster.attrRedisEndpointAddress,
                MERGED_CACHE_PORT: graph_merge_cache.cluster.attrRedisEndpointPort,
                GRAPL_SCHEMA_TABLE: props.schemaTable.schema_table.tableName,
            },
            vpc: props.vpc,
            reads_from: subgraphs_generated.bucket,
            subscribes_to: subgraphs_generated.topic,
            writes_to: props.writesTo,
            version: props.version,
            watchful: props.watchful,
            metric_forwarder: props.metricForwarder,
        });
        props.schemaTable.allowRead(this.service);
        props.dgraphSwarmCluster.allowConnectionsFrom(this.service.event_handler);
    }
}
