import {
    expect as expectCDK,
    matchTemplate,
    MatchStyle,
} from '@aws-cdk/assert';
import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as GraplCdk from '../lib/grapl-cdk-stack';

test('Empty Stack', () => {
    const app = new cdk.App();
    // WHEN
    const stack = new GraplCdk.GraplCdkStack(app, 'MyTestStack', {
        stackName: 'Grapl-Test',
        version: 'latest',
        dgraphInstanceType: new ec2.InstanceType("t3a.medium"),
    });
    // THEN
    expectCDK(stack).to(
        matchTemplate(
            {
                Resources: {},
            },
            MatchStyle.EXACT
        )
    );
});
