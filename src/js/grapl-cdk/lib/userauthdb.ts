import * as cdk from "@aws-cdk/core";
import * as dynamodb from "@aws-cdk/aws-dynamodb";
import * as iam from "@aws-cdk/aws-iam";

import { Service } from "./service"
import { RemovalPolicy } from "@aws-cdk/core";

export class UserAuthDb extends cdk.Construct {
    readonly user_auth_table: dynamodb.Table;

    constructor(
        scope: cdk.Construct,
        id: string,
        // props?: cdk.NestedStackProps

    ) {
        // super(scope, id, props);
        super(scope, id);

        this.user_auth_table = new dynamodb.Table(this, 'user_auth_table', {
            tableName: "user_auth_table",
            partitionKey: {
                name: 'username',
                type: dynamodb.AttributeType.STRING
            },
            serverSideEncryption: true,
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            removalPolicy: RemovalPolicy.DESTROY,
        });
    }

    allowRead(service: Service) {
        this.user_auth_table.grantReadData(service.event_handler);
        this.user_auth_table.grantReadData(service.event_retry_handler);
    }

    allowReadWrite(service: Service) {
        this.user_auth_table.grantReadData(service.event_handler);
        this.user_auth_table.grantReadData(service.event_retry_handler);
    }

    allowReadFromRole(identity: iam.IGrantable) {
        this.user_auth_table.grantReadData(identity);
    }

    allowReadWriteFromRole(identity: iam.IGrantable) {
        this.user_auth_table.grantReadWriteData(identity);
    }
}