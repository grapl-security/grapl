import {
  SynthUtils,
} from '@aws-cdk/assert';
import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as cloudwatch from '@aws-cdk/aws-cloudwatch';
import * as GraplCdk from '../lib/grapl-cdk-stack';

const ENV = { account: '12345', region: 'us-east-1' };
const STACK_NAME = 'Grapl-Test';
const STARTS_WITH_STACK_NAME = new RegExp(`${STACK_NAME}.*`);

class CollectAllConstructs implements cdk.IAspect {
  /**
   * Gathers all constructs from the stack in this property.
   */
  public readonly constructs: cdk.IConstruct[] = []

  public visit(node: cdk.IConstruct): void {
    this.constructs.push(node);
  }

  public getAllOfType<T extends cdk.IConstruct>(type: { new(...args: any[]): T }): T[] {
    const filtered: T[] = [];
    for (const c of this.constructs) {
      if (c instanceof type) {
        filtered.push(c);
      }
    }
    expect(filtered.length).toBeGreaterThan(0);
    return filtered;
  }
}

describe('Standard GraplCdkStack', () => {
  const app = new cdk.App();

  const stack = new GraplCdk.GraplCdkStack(app, 'MyTestStack', {
    stackName: STACK_NAME,
    version: 'latest',
    dgraphInstanceType: new ec2.InstanceType('t3a.medium'),
    env: ENV,
    operationalAlarmsEmail: "fake@fake.domain",
    securityAlarmsEmail: "fake@fake.domain",
  });

  const allConstructs = new CollectAllConstructs();
  cdk.Aspects.of(app).add(allConstructs);
  SynthUtils.synthesize(stack);

  test("fyi you can't test alarms", () => {
    // they just don't show up in the stack, it's a CDK issue
  });

  test('All dashboards have the prefix', () => {
    const constructs = allConstructs.getAllOfType(cloudwatch.Dashboard);

    for (const c of constructs) {
      const name = (c as any).physicalName;
      expect(name).toMatch(STARTS_WITH_STACK_NAME);
    }
  });
});
