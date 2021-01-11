//
// export interface EngagementEdgeProps extends GraplServiceProps {
//     engagement_notebook: EngagementNotebook,
//     edgeApi: apigateway.RestApi,
// }
//
// export class EngagementEdge extends cdk.NestedStack {
//     event_handler: lambda.Function;
//     name: string;
//     apis: WatchedOperation[];
//
//     constructor(scope: cdk.Construct, id: string, props: EngagementEdgeProps) {
//         super(scope, id);
//
//         const ux_bucket = s3.Bucket.fromBucketName(
//             this,
//             'uxBucket',
//             props.prefix.toLowerCase() + '-engagement-ux-bucket'
//         );
//
//         const serviceName = props.prefix + '-EngagementEdge';
//         this.name = id + props.prefix;
//
//         this.event_handler = new lambda.Function(this, 'Handler', {
//             runtime: lambda.Runtime.PYTHON_3_7,
//             handler: `src.engagement_edge.app`,
//             functionName: serviceName + '-Handler',
//             code: lambda.Code.fromAsset(
//                 `./zips/engagement-edge-${props.version}.zip`
//             ),
//             vpc: props.vpc,
//             environment: {
//                 MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
//                 JWT_SECRET_ID: props.jwtSecret.secretArn,
//                 USER_AUTH_TABLE: props.userAuthTable.user_auth_table.tableName,
//                 UX_BUCKET_URL: 'https://' + ux_bucket.bucketRegionalDomainName,
//                 BUCKET_PREFIX: props.prefix,
//             },
//             timeout: cdk.Duration.seconds(25),
//             memorySize: 256,
//             description: props.version,
//         });
//         this.event_handler.currentVersion.addAlias('live');
//
//         props.dgraphSwarmCluster.allowConnectionsFrom(this.event_handler);
//
//         if (props.watchful) {
//             props.watchful.watchLambdaFunction(
//                 this.event_handler.functionName,
//                 this.event_handler
//             );
//         }
//
//         // https://github.com/grapl-security/issue-tracker/issues/115
//         props.engagement_notebook.allowCreatePresignedUrl(this.event_handler);
//
//         if (this.event_handler.role) {
//             props.jwtSecret.grantRead(this.event_handler.role);
//         }
//         props.userAuthTable.allowReadFromRole(this.event_handler);
//
//         const integration = new apigateway.LambdaIntegration(this.event_handler);
//         props.edgeApi.root.addResource('auth').addProxy({
//             defaultIntegration: integration,
//         });
//         this.apis = [];
//         for (const httpMethod of ['POST', 'OPTIONS', 'GET', 'DELETE']) {
//             for (const resourcePath of ['/login', '/checkLogin', '/{proxy+}']) {
//                 this.apis.push({httpMethod, resourcePath});
//                 this.apis.push({httpMethod, resourcePath: '/auth' + resourcePath});
//             }
//         }
//     }
// }
