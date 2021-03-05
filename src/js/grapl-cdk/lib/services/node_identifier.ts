import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3 from '@aws-cdk/aws-s3';
import * as sns from '@aws-cdk/aws-sns';
import { HistoryDb } from '../historydb';
import { EventEmitter } from '../event_emitters';
import { RedisCluster } from '../redis';
import { GraplServiceProps } from '../grapl-cdk-stack';
import { ContainerImage } from "@aws-cdk/aws-ecs";
import { FargateService } from "../fargate_service";
import { GraplS3Bucket } from '../grapl_s3_bucket';
import { SRC_DIR, RUST_DOCKERFILE } from '../dockerfile_paths';

export interface NodeIdentifierProps extends GraplServiceProps {
    writesTo: s3.IBucket;
}

export class NodeIdentifier extends cdk.NestedStack {
    readonly bucket: GraplS3Bucket;
    readonly topic: sns.Topic;
    readonly service: FargateService;

    constructor(parent: cdk.Construct, id: string, props: NodeIdentifierProps) {
        super(parent, id);

        const history_db = new HistoryDb(this, 'HistoryDB', props);

        const service_name = "node-identifier";
        const deployment_name = props.deploymentName.toLowerCase();
        const unid_subgraphs = new EventEmitter(
            this,
            deployment_name + '-unid-subgraphs-generated'
        );
        this.bucket = unid_subgraphs.bucket;
        this.topic = unid_subgraphs.topic;

        const event_cache = new RedisCluster(
            this,
            'NodeIdentifierRetryCache',
            props
        );
        event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        this.service = new FargateService(this, service_name, {
            deploymentName: props.deploymentName,
            environment: {
                RUST_LOG: props.nodeIdentifierLogLevel,
                EVENT_CACHE_CLUSTER_ADDRESS: event_cache.address,
                RETRY_IDENTITY_CACHE_ADDR:
                event_cache.cluster.attrRedisEndpointAddress,
                RETRY_IDENTITY_CACHE_PORT:
                event_cache.cluster.attrRedisEndpointPort,
                STATIC_MAPPING_TABLE: history_db.static_mapping_table.tableName,
                DYNAMIC_SESSION_TABLE:
                history_db.dynamic_session_table.tableName,
                PROCESS_HISTORY_TABLE: history_db.proc_history.tableName,
                FILE_HISTORY_TABLE: history_db.file_history.tableName,
                INBOUND_CONNECTION_HISTORY_TABLE:
                history_db.inbound_connection_history.tableName,
                OUTBOUND_CONNECTION_HISTORY_TABLE:
                history_db.outbound_connection_history.tableName,
                NETWORK_CONNECTION_HISTORY_TABLE:
                history_db.network_connection_history.tableName,
                IP_CONNECTION_HISTORY_TABLE:
                history_db.ip_connection_history.tableName,
                ASSET_ID_MAPPINGS: history_db.asset_history.tableName,
            },
            vpc: props.vpc,
            eventEmitter: unid_subgraphs,
            writesTo: props.writesTo,
            version: props.version,
            watchful: props.watchful,
            serviceImage: ContainerImage.fromAsset(SRC_DIR, {
                target: "node-identifier-deploy",
                buildArgs: {
                    "CARGO_PROFILE": "debug"
                },
                file: RUST_DOCKERFILE,
            }),
            retryServiceImage: ContainerImage.fromAsset(SRC_DIR, {
                target: "node-identifier-retry-handler-deploy",
                buildArgs: {
                    "CARGO_PROFILE": "debug"
                },
                file: RUST_DOCKERFILE,
            }),
            command: ["/node-identifier"],
            retryCommand: ["/node-identifier-retry-handler"],
            metric_forwarder: props.metricForwarder,
        });

        this.service.service.cluster.connections.allowToAnyIpv4(
            ec2.Port.tcp(parseInt(event_cache.cluster.attrRedisEndpointPort))
        );

        history_db.allowReadWrite2(this.service);
    }
}
