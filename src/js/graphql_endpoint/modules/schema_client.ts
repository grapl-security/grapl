import * as dynamodb from "@aws-sdk/client-dynamodb";

class SchemaClient {
    // Also compare with get_r_edges_from_dynamodb
    readonly client: dynamodb.DynamoDB;
    readonly schemaTableName: string;

    constructor() {
        this.client = new dynamodb.DynamoDB({});
        this.schemaTableName = process.env.GRAPL_SCHEMA_TABLE;
    }

    async getSchemas() {
        const command = new dynamodb.ScanCommand({
            TableName: this.schemaTableName
        });
        const scan = await this.client.send(command);
    }
}