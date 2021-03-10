import * as cdk from '@aws-cdk/core';
import * as dynamodb from '@aws-cdk/aws-dynamodb';

import { GraplServiceProps } from './grapl-cdk-stack';
import { Service } from './service';
import { RemovalPolicy } from '@aws-cdk/core';
import { FargateService } from "./fargate_service";

function create_table(scope: cdk.Construct, name: string) {
    return new dynamodb.Table(scope, name, {
        tableName: name,
        partitionKey: {
            name: 'pseudo_key',
            type: dynamodb.AttributeType.STRING,
        },
        sortKey: {
            name: 'create_time',
            type: dynamodb.AttributeType.NUMBER,
        },
        billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        removalPolicy: RemovalPolicy.DESTROY,
    });
}

export class HistoryDb extends cdk.Construct {
    readonly proc_history: dynamodb.Table;
    readonly file_history: dynamodb.Table;
    readonly outbound_connection_history: dynamodb.Table;
    readonly inbound_connection_history: dynamodb.Table;
    readonly network_connection_history: dynamodb.Table;
    readonly ip_connection_history: dynamodb.Table;
    readonly asset_history: dynamodb.Table;
    readonly dynamic_session_table: dynamodb.Table;
    readonly static_mapping_table: dynamodb.Table;

    constructor(scope: cdk.Construct, id: string, props: GraplServiceProps) {
        super(scope, id);

        this.proc_history = create_table(
            this,
            props.deploymentName + '-process_history_table'
        );
        this.file_history = create_table(
            this,
            props.deploymentName + '-file_history_table'
        );
        this.outbound_connection_history = create_table(
            this,
            props.deploymentName + '-outbound_connection_history_table'
        );
        this.inbound_connection_history = create_table(
            this,
            props.deploymentName + '-inbound_connection_history_table'
        );
        this.network_connection_history = create_table(
            this,
            props.deploymentName + '-network_connection_history_table'
        );
        this.ip_connection_history = create_table(
            this,
            props.deploymentName + '-ip_connection_history_table'
        );
        this.dynamic_session_table = create_table(
            this,
            props.deploymentName + '-dynamic_session_table'
        );

        this.asset_history = new dynamodb.Table(this, 'AssetIdMappings', {
            tableName: props.deploymentName + '-asset_id_mappings',
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING,
            },
            sortKey: {
                name: 'c_timestamp',
                type: dynamodb.AttributeType.NUMBER,
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });

        this.static_mapping_table = new dynamodb.Table(
            this,
            'StaticMappingTable',
            {
                tableName: props.deploymentName + '-static_mapping_table',
                partitionKey: {
                    name: 'pseudo_key',
                    type: dynamodb.AttributeType.STRING,
                },
                billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
                removalPolicy: RemovalPolicy.DESTROY,
            }
        );
    }

    allowReadWrite2(service: FargateService) {
        const tables = [
            this.proc_history,
            this.file_history,
            this.outbound_connection_history,
            this.inbound_connection_history,
            this.network_connection_history,
            this.ip_connection_history,
            this.asset_history,
            this.static_mapping_table,
            this.dynamic_session_table,
        ];
        for (const table of tables) {
            table.grantReadWriteData(service.service.taskDefinition.taskRole);	
        }
    }

    allowReadWrite(service: Service) {
        this.proc_history.grantReadWriteData(service.event_handler);
        this.file_history.grantReadWriteData(service.event_handler);
        this.outbound_connection_history.grantReadWriteData(
            service.event_handler
        );
        this.inbound_connection_history.grantReadWriteData(
            service.event_handler
        );
        this.network_connection_history.grantReadWriteData(
            service.event_handler
        );
        this.ip_connection_history.grantReadWriteData(service.event_handler);
        this.asset_history.grantReadWriteData(service.event_handler);
        this.static_mapping_table.grantReadWriteData(service.event_handler);
        this.dynamic_session_table.grantReadWriteData(service.event_handler);

        this.proc_history.grantReadWriteData(service.event_retry_handler);
        this.file_history.grantReadWriteData(service.event_retry_handler);
        this.outbound_connection_history.grantReadWriteData(
            service.event_retry_handler
        );
        this.inbound_connection_history.grantReadWriteData(
            service.event_retry_handler
        );
        this.network_connection_history.grantReadWriteData(
            service.event_retry_handler
        );
        this.ip_connection_history.grantReadWriteData(
            service.event_retry_handler
        );
        this.asset_history.grantReadWriteData(service.event_retry_handler);
        this.static_mapping_table.grantReadWriteData(
            service.event_retry_handler
        );
        this.dynamic_session_table.grantReadWriteData(
            service.event_retry_handler
        );
    }
}
