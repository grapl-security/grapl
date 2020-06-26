import * as cdk from "@aws-cdk/core";
import * as dynamodb from "@aws-cdk/aws-dynamodb";

import { GraplServiceProps } from './grapl-cdk-stack';
import { Service } from "./service";
import { RemovalPolicy } from "@aws-cdk/core";

function create_table(
    scope: cdk.Construct,
    name: string,
) {
    return new dynamodb.Table(scope, name, {
        tableName: name,
        partitionKey: {
            name: 'pseudo_key',
            type: dynamodb.AttributeType.STRING
        },
        sortKey: {
            name: 'create_time',
            type: dynamodb.AttributeType.NUMBER
        },
        billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        removalPolicy: RemovalPolicy.DESTROY,
    });
}

export class HistoryDb extends cdk.Construct {

    readonly ProcessHistoryTable: dynamodb.Table;
    readonly FileHistoryTable: dynamodb.Table;
    readonly OutboundConnectionHistoryTable: dynamodb.Table;
    readonly InboundConnectionHistoryTable: dynamodb.Table;
    readonly NetworkConnectionHistoryTable: dynamodb.Table;
    readonly IpConnectionHistoryTable: dynamodb.Table;
    readonly AssetHistoryTable: dynamodb.Table;
    readonly NodeIdRetryTable: dynamodb.Table;
    readonly DynamicSessionTable: dynamodb.Table;
    readonly static_mapping_table: dynamodb.Table;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: GraplServiceProps,
    ) {
        super(scope, id);

        this.ProcessHistoryTable = create_table(this, props.prefix + '-process_history_table');
        this.FileHistoryTable = create_table(this, props.prefix + '-file_history_table');
        this.OutboundConnectionHistoryTable = create_table(this, props.prefix + '-outbound_connection_history_table');
        this.InboundConnectionHistoryTable = create_table(this, props.prefix + '-inbound_connection_history_table');
        this.NetworkConnectionHistoryTable = create_table(this, props.prefix + '-network_connection_history_table');
        this.IpConnectionHistoryTable = create_table(this, props.prefix + '-ip_connection_history_table');
        this.DynamicSessionTable = create_table(this, props.prefix + '-dynamic_session_table');

        this.AssetHistoryTable = new dynamodb.Table(this, 'AssetIdMappings', {
            tableName: props.prefix + '-asset_id_mappings',
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            sortKey: {
                name: 'c_timestamp',
                type: dynamodb.AttributeType.NUMBER
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });

        this.static_mapping_table = new dynamodb.Table(this, 'StaticMappingTable', {
            tableName:  props.prefix + '-static_mapping_table',
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });

        this.NodeIdRetryTable = new dynamodb.Table(this, 'NodeIdRetryTable', {
            tableName:  props.prefix + '-node_id_retry_table',
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            timeToLiveAttribute: 'ttl_ts',
            removalPolicy: RemovalPolicy.DESTROY,
        });
    }

    allowReadWrite(service: Service) {
        this.ProcessHistoryTable.grantReadWriteData(service.event_handler);
        this.FileHistoryTable.grantReadWriteData(service.event_handler);
        this.OutboundConnectionHistoryTable.grantReadWriteData(service.event_handler);
        this.InboundConnectionHistoryTable.grantReadWriteData(service.event_handler);
        this.NetworkConnectionHistoryTable.grantReadWriteData(service.event_handler);
        this.IpConnectionHistoryTable.grantReadWriteData(service.event_handler);
        this.AssetHistoryTable.grantReadWriteData(service.event_handler);
        this.NodeIdRetryTable.grantReadWriteData(service.event_handler);
        this.static_mapping_table.grantReadWriteData(service.event_handler);
        this.DynamicSessionTable.grantReadWriteData(service.event_handler);

        this.ProcessHistoryTable.grantReadWriteData(service.event_retry_handler);
        this.FileHistoryTable.grantReadWriteData(service.event_retry_handler);
        this.OutboundConnectionHistoryTable.grantReadWriteData(service.event_retry_handler);
        this.InboundConnectionHistoryTable.grantReadWriteData(service.event_retry_handler);
        this.NetworkConnectionHistoryTable.grantReadWriteData(service.event_retry_handler);
        this.IpConnectionHistoryTable.grantReadWriteData(service.event_retry_handler);
        this.AssetHistoryTable.grantReadWriteData(service.event_retry_handler);
        this.NodeIdRetryTable.grantReadWriteData(service.event_retry_handler);
        this.static_mapping_table.grantReadWriteData(service.event_retry_handler);
        this.DynamicSessionTable.grantReadWriteData(service.event_retry_handler);
    }
}