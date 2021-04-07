import * as cdk from '@aws-cdk/core';
import { DbTable } from './dynamoDbTable';
import * as dynamodb from '@aws-cdk/aws-dynamodb';
import { Service } from './service';
import * as iam from '@aws-cdk/aws-iam';
import { FargateService } from "./fargate_service";

export interface SchemaDbProps {
    edges_table_name: string;
    properties_table_name: string;
}

export class SchemaDb extends cdk.Construct {
    readonly schema_table: DbTable;
    readonly schema_properties_table: DbTable;

    constructor(scope: cdk.Construct, id: string, props: SchemaDbProps) {
        super(scope, id);

        this.schema_table = new DbTable(this, 'SchemaDbTable', {
            tableName: props.edges_table_name,
            table: 'schema_db_table',
            partitionKey: {
                name: 'f_edge',
                type: dynamodb.AttributeType.STRING,
            },
            tableReference: 'SchemaDbTable',
        });

        this.schema_properties_table = new DbTable(
            this,
            'SchemaPropertiesDbTable',
            {
                tableName: props.properties_table_name,
                table: 'schema_properties_db_table',
                partitionKey: {
                    name: 'node_type',
                    type: dynamodb.AttributeType.STRING,
                },
                tableReference: 'SchemaPropertiesDbTable',
            }
        );
    }

    allowRead(service: Service|FargateService) {
        for (const table of [this.schema_table, this.schema_properties_table]) {
            if (service instanceof FargateService) {
                table.table.grantReadData(service.service.taskDefinition.taskRole);
                table.table.grantReadData(service.retryService.taskDefinition.taskRole);
            } else {
                table.table.grantReadData(service.event_handler);
                table.table.grantReadData(service.event_retry_handler);
            }
        }
    }


    allowReadWrite(service: Service) {
        for (const table of [this.schema_table, this.schema_properties_table]) {
            table.table.grantReadWriteData(service.event_handler);
            table.table.grantReadWriteData(service.event_retry_handler);
        }
    }

    allowReadFromRole(identity: iam.IGrantable) {
        for (const table of [this.schema_table, this.schema_properties_table]) {
            table.table.grantReadData(identity);
        }
    }

    allowReadWriteFromRole(identity: iam.IGrantable) {
        for (const table of [this.schema_table, this.schema_properties_table]) {
            table.table.grantReadWriteData(identity);
        }
    }
}
