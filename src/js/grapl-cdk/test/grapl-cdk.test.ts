import {
    expect as expectCDK,
    matchTemplate,
    MatchStyle,
} from '@aws-cdk/assert';
import * as cdk from '@aws-cdk/core';
import * as GraplCdk from '../lib/grapl-cdk-stack';

test('Empty Stack', () => {
    const app = new cdk.App();
    // WHEN
    const stack = new GraplCdk.GraplCdkStack(app, 'MyTestStack', {
        stackName: 'Grapl-Test',
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
