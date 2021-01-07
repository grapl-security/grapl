import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3 from '@aws-cdk/aws-s3';
import * as sns from '@aws-cdk/aws-sns';
import { Service } from '../service';
import { HistoryDb } from '../historydb';
import { EventEmitter } from '../event_emitters';
import { RedisCluster } from '../redis';
import { GraplServiceProps } from '../grapl-cdk-stack';

export interface NodeIdentifierProps extends GraplServiceProps {
    writesTo: s3.IBucket;
}

export class NodeIdentifier extends cdk.NestedStack {
    readonly bucket: s3.Bucket;
    readonly topic: sns.Topic;
    readonly service: Service;

    constructor(parent: cdk.Construct, id: string, props: NodeIdentifierProps) {
        super(parent, id);

        const history_db = new HistoryDb(this, 'HistoryDB', props);

        const bucket_prefix = props.prefix.toLowerCase();
        const unid_subgraphs = new EventEmitter(
            this,
            bucket_prefix + '-unid-subgraphs-generated'
        );
        this.bucket = unid_subgraphs.bucket;
        this.topic = unid_subgraphs.topic;

        const retry_identity_cache = new RedisCluster(
            this,
            'NodeIdentifierRetryCache',
            props
        );
        retry_identity_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        this.service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                BUCKET_PREFIX: bucket_prefix,
                RETRY_IDENTITY_CACHE_ADDR: retry_identity_cache.cluster.attrRedisEndpointAddress,
                RETRY_IDENTITY_CACHE_PORT: retry_identity_cache.cluster.attrRedisEndpointPort,
                STATIC_MAPPING_TABLE: history_db.static_mapping_table.tableName,
                DYNAMIC_SESSION_TABLE: history_db.dynamic_session_table.tableName,
                PROCESS_HISTORY_TABLE: history_db.proc_history.tableName,
                FILE_HISTORY_TABLE: history_db.file_history.tableName,
                INBOUND_CONNECTION_HISTORY_TABLE: history_db.inbound_connection_history.tableName,
                OUTBOUND_CONNECTION_HISTORY_TABLE: history_db.outbound_connection_history.tableName,
                NETWORK_CONNECTION_HISTORY_TABLE: history_db.network_connection_history.tableName,
                IP_CONNECTION_HISTORY_TABLE: history_db.ip_connection_history.tableName,
                ASSET_ID_MAPPINGS: history_db.asset_history.tableName,
            },
            vpc: props.vpc,
            reads_from: unid_subgraphs.bucket,
            subscribes_to: unid_subgraphs.topic,
            writes_to: props.writesTo,
            retry_code_name: 'node-identifier-retry-handler',
            version: props.version,
            watchful: props.watchful,
            metric_forwarder: props.metricForwarder,
        });

        history_db.allowReadWrite(this.service);

        this.service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(
                parseInt(retry_identity_cache.cluster.attrRedisEndpointPort)
            )
        );

        this.service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(
                parseInt(retry_identity_cache.cluster.attrRedisEndpointPort)
            )
        );

        this.service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(443),
            'Allow outbound to S3'
        );
        this.service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(443),
            'Allow outbound to S3'
        );
    }
}
