import * as cdk from '@aws-cdk/core';
import { DbTable } from './dyanamoDbTable';
import * as dynamodb from '@aws-cdk/aws-dynamodb';

export interface SchemaDbProps {
    deploymentName: string;
}

export class SchemaDb extends DbTable {
    constructor(scope: cdk.Construct, id: string, props: SchemaDbProps) {
        super(scope, id, {
            tableName: props.deploymentName + '-grapl_schema_table',
            table: 'schema_table',
            partitionKey: {
                name: 'f_edge',
                type: dynamodb.AttributeType.STRING,
            },
            tableReference: 'SchemaDbTable',
        });
    }
}
