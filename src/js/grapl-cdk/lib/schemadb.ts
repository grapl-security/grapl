import * as cdk from '@aws-cdk/core';
import * as dynamodb from '@aws-cdk/aws-dynamodb';
import * as iam from '@aws-cdk/aws-iam';

import { Service } from './service';
import { RemovalPolicy } from '@aws-cdk/core';
import { FargateService } from "./fargate_service";

export interface SchemaDbProps {
    edges_table_name: string;
    properties_table_name: string;
}

export class SchemaDb extends cdk.Construct {
    readonly schema_table: dynamodb.Table;
    readonly schema_properties_table: dynamodb.Table;

    constructor(scope: cdk.Construct, id: string, props: SchemaDbProps) {
        super(scope, id);

        this.schema_table = new dynamodb.Table(this, 'SchemaDbTable', {
            tableName: props.edges_table_name,
            partitionKey: {
                name: 'f_edge',
                type: dynamodb.AttributeType.STRING,
            },
            serverSideEncryption: true,
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });

        this.schema_properties_table = new dynamodb.Table(this, 'SchemaPropertiesDbTable', {
            tableName: props.properties_table_name,
            partitionKey: {
                name: 'node_type',
                type: dynamodb.AttributeType.STRING,
            },
            serverSideEncryption: true,
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });
    }

    allowRead(service: Service|FargateService) {
        for (const table of [this.schema_table, this.schema_properties_table]) {
            if (service instanceof FargateService) {
                table.grantReadData(service.service.taskDefinition.taskRole);
                table.grantReadData(service.retryService.taskDefinition.taskRole);
            } else {
                table.grantReadData(service.event_handler);
                table.grantReadData(service.event_retry_handler);
            }
        }
    }

    allowReadWrite(service: Service) {
        for (const table of [this.schema_table, this.schema_properties_table]) {
            table.grantReadWriteData(service.event_handler);
            table.grantReadWriteData(service.event_retry_handler);
        }
    }

    allowReadFromRole(identity: iam.IGrantable) {
        for (const table of [this.schema_table, this.schema_properties_table]) {
            table.grantReadData(identity);
        }
    }

    allowReadWriteFromRole(identity: iam.IGrantable) {
        for (const table of [this.schema_table, this.schema_properties_table]) {
            table.grantReadWriteData(identity);
        }
    }
}
