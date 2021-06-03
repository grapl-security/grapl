import * as cdk from "@aws-cdk/core";
import * as dynamodb from "@aws-cdk/aws-dynamodb";

import { GraplServiceProps } from "./grapl-cdk-stack";
import { Service } from "./service";
import { RemovalPolicy } from "@aws-cdk/core";
import { FargateService } from "./fargate_service";

function create_table(scope: cdk.Construct, name: string) {
    return new dynamodb.Table(scope, name, {
        tableName: name,
        partitionKey: {
            name: "pseudo_key",
            type: dynamodb.AttributeType.STRING,
        },
        sortKey: {
            name: "create_time",
            type: dynamodb.AttributeType.NUMBER,
        },
        billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        removalPolicy: RemovalPolicy.DESTROY,
    });
}

export class HistoryDb extends cdk.Construct {
    readonly dynamic_session_table: dynamodb.Table;

    constructor(scope: cdk.Construct, id: string, props: GraplServiceProps) {
        super(scope, id);
        this.dynamic_session_table = create_table(
            this,
            props.deploymentName + "-dynamic_session_table"
        );
    }

    allowReadWrite2(service: FargateService) {
        const tables = [
            this.dynamic_session_table,
        ];
        for (const table of tables) {
            table.grantReadWriteData(service.service.taskDefinition.taskRole);
        }
    }

    allowReadWrite(service: Service) {
        this.dynamic_session_table.grantReadWriteData(service.event_handler);
        this.dynamic_session_table.grantReadWriteData(service.event_retry_handler);
    }
}
