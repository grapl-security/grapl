import * as dynamodb from "@aws-sdk/client-dynamodb";
import * as util_dynamodb from "@aws-sdk/util-dynamodb";

import * as aws_types from "@aws-sdk/types";

function getDynamodbClient() {
    if ("GRAPL_AWS_ENDPOINT" in process.env) {
        // Running locally
        const credentials: aws_types.Credentials = {
            accessKeyId: process.env.GRAPL_AWS_ACCESS_KEY_ID,
            secretAccessKey: process.env.GRAPL_AWS_ACCESS_KEY_SECRET,
        };
        return new dynamodb.DynamoDB({
            endpoint: process.env.GRAPL_AWS_ENDPOINT,
            credentials: credentials,
        });
    } else {
        // Running on AWS
        return new dynamodb.DynamoDB({});
    }
}

export interface Schema {
    readonly type_definition: {
        readonly properties: SchemaProperty[];
    };
    readonly node_type: string;
    readonly display_property: string;
}

export interface SchemaProperty {
    readonly name: string;
    readonly primitive: string; // "Int" | "Str" | "Bool" or another Schema's name
    readonly is_set: boolean;
}

export class SchemaClient {
    // Also compare with get_r_edges_from_dynamodb
    readonly client: dynamodb.DynamoDB;
    readonly schemaTableName: string;

    constructor() {
        this.client = getDynamodbClient();
        this.schemaTableName = process.env.GRAPL_SCHEMA_PROPERTIES_TABLE;
    }

    async getSchemas(): Promise<Schema[]> {
        const command = new dynamodb.ScanCommand({
            TableName: this.schemaTableName,
        });
        try {
            const scan = await this.client.send(command);
            const schemas = scan.Items.map(
                (item) => util_dynamodb.unmarshall(item) as Schema
            );
            return schemas;
        } catch (e) {
            console.error("Get Schemas failure", e);
            throw e;
        }
    }
}
