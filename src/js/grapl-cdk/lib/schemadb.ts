import * as cdk from '@aws-cdk/core';
import * as dynamodb from '@aws-cdk/aws-dynamodb';
import * as iam from '@aws-cdk/aws-iam';

import { Service } from './service';
import { RemovalPolicy } from '@aws-cdk/core';
import { FargateService } from "./fargate_service";

export interface SchemaDbProps {
    table_name: string;
}

export class SchemaDb extends cdk.Construct {
    readonly schema_table: dynamodb.Table;

    constructor(scope: cdk.Construct, id: string, props: SchemaDbProps) {
        super(scope, id);

        this.schema_table = new dynamodb.Table(this, 'SchemaDbTable', {
            tableName: props.table_name,
            partitionKey: {
                name: 'f_edge',
                type: dynamodb.AttributeType.STRING,
            },
            serverSideEncryption: true,
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });
    }

    allowRead(service: Service|FargateService) {
        if (service instanceof FargateService) {
            for (const taskRole of service.taskRoles()) {
                this.schema_table.grantReadData(taskRole);
            }
        } else {
            this.schema_table.grantReadData(service.event_handler);
            this.schema_table.grantReadData(service.event_retry_handler);
        }
    }

    allowReadWrite(service: Service) {
        this.schema_table.grantReadWriteData(service.event_handler);
        this.schema_table.grantReadWriteData(service.event_retry_handler);
    }

    allowReadFromRole(identity: iam.IGrantable) {
        this.schema_table.grantReadData(identity);
    }

    allowReadWriteFromRole(identity: iam.IGrantable) {
        this.schema_table.grantReadWriteData(identity);
    }
}
