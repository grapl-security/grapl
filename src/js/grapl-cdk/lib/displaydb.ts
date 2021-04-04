import * as cdk from '@aws-cdk/core';
import { DbTable } from './dyanamoDbTable';
import * as dynamodb from '@aws-cdk/aws-dynamodb';

export interface DisplayPropertyDbProps {
    deploymentName: string;
}

export class DisplayPropertyDb extends DbTable {
    constructor(scope: cdk.Construct, id: string, props: DisplayPropertyDbProps) {
        super(scope, id, {
            tableName: props.deploymentName + '-grapl_display_table',
            table: 'display_table',
            partitionKey: {
                name: 'node_type',
                type: dynamodb.AttributeType.STRING,
            },
            tableReference: 'DisplayPropertyDbTable',
        });
    }
}
