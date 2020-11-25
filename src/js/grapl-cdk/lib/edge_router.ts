import * as cdk from '@aws-cdk/core';
import * as apigateway from '@aws-cdk/aws-apigateway';
import * as lambda from '@aws-cdk/aws-lambda';
import {GraplServiceProps} from "./grapl-cdk-stack";

export interface RouteTarget {
    routeName: string,
    routeFilter: string,
    target: lambda.Function,
    port: number,
}


export interface EdgeRouter extends GraplServiceProps {
    // Array of route to API
    routers: RouteTarget[],
}


export class EdgeRouter extends cdk.NestedStack {
    constructor(scope: cdk.Construct, id: string, props: EdgeRouter)
    {
        super(scope, id);

        const api = new apigateway.RestApi(this, 'hello-api', { });
        api.restApiId
        // for (const router of props.routers) {
        //     const integration = new apigateway.LambdaIntegration(router.target);
        //     const route = api.root.addResource(router.routeFilter,);
        //
        //     route.addMethod(
        //         'ANY', integration,
        //     );
        //
        // }
    }
}