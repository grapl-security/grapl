import { IAspect, IConstruct } from '@aws-cdk/core';
import apigw = require('@aws-cdk/aws-apigateway');
import dynamodb = require('@aws-cdk/aws-dynamodb');
import lambda = require('@aws-cdk/aws-lambda');

export interface WatchfulAspectProps {
  /**
   * Automatically watch API Gateway APIs in the scope.
   * @default true
   */
  readonly apiGateway?: boolean;

  /**
   * Automatically watch all Amazon DynamoDB tables in the scope.
   * @default true
   */
  readonly dynamodb?: boolean;

  /**
   * Automatically watch AWS Lambda functions in the scope.
   * @default true
   */
  readonly lambda?: boolean;
}

/**
 * A CDK aspect that can automatically watch all resources within a scope.
 */
export class WatchfulAspect implements IAspect {
  /**
   * Defines a watchful aspect
   * @param watchful The watchful to add those resources to
   * @param props Options
   */
  constructor(private readonly watchful: Watchful, private readonly props: WatchfulAspectProps = { }) {

  }

  public visit(node: IConstruct): void {
    const watchApiGateway = this.props.apiGateway === undefined ? true : this.props.apiGateway;
    const watchDynamo = this.props.dynamodb === undefined ? true : this.props.dynamodb;
    const watchLambda = this.props.lambda === undefined ? true : this.props.lambda;

    if (watchApiGateway && node instanceof apigw.RestApi) {
      this.watchful.watchApiGateway(node.node.path, node);
    }

    if (watchDynamo && node instanceof dynamodb.Table) {
      this.watchful.watchDynamoTable(node.node.path, node);
    }

    if (watchLambda && node instanceof lambda.Function) {
      this.watchful.watchLambdaFunction(node.node.path, node);
    }
  }
}

import { Watchful } from './watchful';
