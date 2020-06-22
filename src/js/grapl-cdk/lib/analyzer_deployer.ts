import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import {Runtime} from "@aws-cdk/aws-lambda";
import * as apigateway from "@aws-cdk/aws-apigateway";
import * as s3 from "@aws-cdk/aws-s3";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as secretsmanager from '@aws-cdk/aws-secretsmanager';

class AnalyzerDeployer extends cdk.NestedStack {
    event_handler: lambda.Function;
    integration: apigateway.LambdaRestApi;
    name: string;
    integrationName: string;

    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        jwt_secret: secretsmanager.Secret,
        analyzer_bucket: s3.IBucket,
        vpc: ec2.Vpc,
    ) {
        super(scope, id);

        const environment = {} as any;
        if (process.env.GITHUB_SHARED_SECRET) {
            environment.GITHUB_SHARED_SECRET = process.env.GITHUB_SHARED_SECRET;
        }
        if (process.env.GITHUB_ACCESS_TOKEN) {
            environment.GITHUB_ACCESS_TOKEN = process.env.GITHUB_ACCESS_TOKEN;
        }
        if (process.env.GITHUB_REPOSITORY_NAME) {
            environment.GITHUB_REPOSITORY_NAME = process.env.GITHUB_REPOSITORY_NAME;
        }

        this.event_handler = new lambda.Function(
            this, `${id}-handler`, {
                runtime: Runtime.PYTHON_3_7,
                handler: `main.lambda_handler`,
                code: lambda.Code.fromAsset(`./analyzer_deployer.zip`),
                vpc: vpc,
                environment: {
                    ...environment,
                    "BUCKET_PREFIX": prefix,
                },
                timeout: cdk.Duration.seconds(25),
                memorySize: 256,
            }
        );

        new apigateway.LambdaRestApi(this, id, {
            handler: this.event_handler,
        });

        // We need these permissions so that we can:
        // * List and read analyzers, to display on the frontend
        // * Upload new analyzers
        // * Delete deprecated analyzers
        analyzer_bucket.grantDelete(this.event_handler);
        analyzer_bucket.grantReadWrite(this.event_handler);
    }
}

export default AnalyzerDeployer;