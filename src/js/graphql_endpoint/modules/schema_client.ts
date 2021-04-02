import * as dynamodb from "@aws-sdk/client-dynamodb";
import * as util_dynamodb from "@aws-sdk/util-dynamodb";

import * as aws_types from "@aws-sdk/types";

function getDynamodbClient() {
    if ("DYNAMODB_ENDPOINT" in process.env) {
        // Running locally
        const credentials: aws_types.Credentials = {
            accessKeyId: process.env.DYNAMODB_ACCESS_KEY_ID,
            secretAccessKey: process.env.DYNAMODB_ACCESS_KEY_SECRET,
        }
        return new dynamodb.DynamoDB({
            endpoint: process.env.DYNAMODB_ENDPOINT,
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
    }
    readonly node_type: string;
}

export interface SchemaProperty {
    readonly name: string;
    readonly primitive: "Int" | "Str" | "Bool";  // see PropPrimitive in python
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
            TableName: this.schemaTableName
        });
        try { 
            const scan = await this.client.send(command);
            const schemas = scan.Items.map((item) => util_dynamodb.unmarshall(item) as Schema);
            return schemas;
        } catch(e) {
            console.error("Get Schemas failure", e);
            throw e;
        }
    }
}