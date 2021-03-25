import * as cdk from '@aws-cdk/core';
import * as dynamodb from '@aws-cdk/aws-dynamodb';
import * as iam from '@aws-cdk/aws-iam';

import { Service } from './service';
import { RemovalPolicy } from '@aws-cdk/core';
import { FargateService } from "./fargate_service";

export interface DbTableProps {
    tableName: string;
    table: string; 
    tableReference: string; 
    partitionKey: {
        name: string,
        type: dynamodb.AttributeType,
    }; 
    sortKey?: {
        name: string, 
        type: dynamodb.AttributeType, 
    } | undefined; 
}

export class DbTable extends cdk.Construct {
    readonly table: dynamodb.Table; 

    constructor(scope: cdk.Construct, id: string, props: DbTableProps) { 
        super(scope, id);

        this.table = new dynamodb.Table(this, props.tableReference, {
            tableName: props.tableName,
            partitionKey: props.partitionKey,
            sortKey: props.sortKey,
            serverSideEncryption: true,
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });
    }

    allowRead(service: Service|FargateService) {
        if (service instanceof FargateService) {
            this.table.grantReadData(service.service.taskDefinition.taskRole);
            this.table.grantReadData(service.retryService.taskDefinition.taskRole);
        } else {
            this.table.grantReadData(service.event_handler);
            this.table.grantReadData(service.event_retry_handler);
        }
    }

    allowReadWrite(service: Service) {
        this.table.grantReadWriteData(service.event_handler);
        this.table.grantReadWriteData(service.event_retry_handler);
    }

    allowReadFromRole(identity: iam.IGrantable) {
        this.table.grantReadData(identity);
    }

    allowReadWriteFromRole(identity: iam.IGrantable) {
        this.table.grantReadWriteData(identity);
    }
}
