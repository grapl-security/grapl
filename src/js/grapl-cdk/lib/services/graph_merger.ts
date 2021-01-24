import * as path from 'path';
import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3 from '@aws-cdk/aws-s3';
import {EventEmitter} from '../event_emitters';
import {RedisCluster} from '../redis';
import {GraplServiceProps} from '../grapl-cdk-stack';
import {SchemaDb} from '../schemadb';
import {ContainerImage} from "@aws-cdk/aws-ecs";
import {FargateService} from "../fargate_service";

export interface GraphMergerProps extends GraplServiceProps {
    writesTo: s3.IBucket;
    schemaTable: SchemaDb;
}

export class GraphMerger extends cdk.NestedStack {
    readonly bucket: s3.Bucket;
    readonly service: FargateService

    constructor(scope: cdk.Construct, id: string, props: GraphMergerProps) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const subgraphs_generated = new EventEmitter(
            this,
            bucket_prefix + '-subgraphs-generated'
        );
        this.bucket = subgraphs_generated.bucket;

        const event_cache = new RedisCluster(
            this,
            'GraphMergerMergedCache',
            props
        );

        event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        this.service = new FargateService(this, id, {
            prefix: props.prefix,
            environment: {
                EVENT_CACHE_CLUSTER_ADDRESS: event_cache.address,
                RUST_LOG: props.graphMergerLogLevel,
                BUCKET_PREFIX: bucket_prefix,
                SUBGRAPH_MERGED_BUCKET: props.writesTo.bucketName,
                MG_ALPHAS: 'http://' + props.dgraphSwarmCluster.alphaHostPort(),
                MERGED_CACHE_ADDR: event_cache.cluster.attrRedisEndpointAddress,
                MERGED_CACHE_PORT: event_cache.cluster.attrRedisEndpointPort,
                GRAPL_SCHEMA_TABLE: props.schemaTable.schema_table.tableName,
            },
            vpc: props.vpc,
            eventEmitter: subgraphs_generated,
            writesTo: props.writesTo,
            version: props.version,
            watchful: props.watchful,
            serviceImage: ContainerImage.fromAsset(path.join(__dirname, '../../../src/rust/'), {
                target: "graph-merger-deploy",
                buildArgs: {
                    "release_target": "debug"
                },
                file: "Dockerfile",
            }),
            command: ["/graph-merger"],
            // metric_forwarder: props.metricForwarder,
        });

        // probably only needs 9080
        this.service.service.cluster.connections.allowToAnyIpv4(
            ec2.Port.allTcp()
        );
        // probably only needs 9080
        this.service.retryService.cluster.connections.allowToAnyIpv4(
            ec2.Port.allTcp()
        );
        props.schemaTable.allowRead(this.service);
        props.dgraphSwarmCluster.allowConnectionsFrom(this.service.service.cluster);
        props.dgraphSwarmCluster.allowConnectionsFrom(this.service.retryService.cluster);
    }
}
