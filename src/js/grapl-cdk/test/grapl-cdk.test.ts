import { SynthUtils } from "@aws-cdk/assert";
import * as cdk from "@aws-cdk/core";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as cloudwatch from "@aws-cdk/aws-cloudwatch";
import * as GraplCdk from "../lib/grapl-cdk-stack";
import { validateDeploymentName } from "../bin/deployment_parameters";

const ENV = { account: "12345", region: "us-east-1" };
const STACK_NAME = "Grapl-Test";
const STARTS_WITH_STACK_NAME = new RegExp(`${STACK_NAME}.*`);

class CollectAllConstructs implements cdk.IAspect {
    /**
     * Gathers all constructs from the stack in this property.
     */
    public readonly constructs: cdk.IConstruct[] = [];

    public visit(node: cdk.IConstruct): void {
        this.constructs.push(node);
    }

    public getAllOfType<T extends cdk.IConstruct>(type: {
        new (...args: any[]): T;
    }): T[] {
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

describe("Standard GraplCdkStack", () => {
    const app = new cdk.App();

    const stack = new GraplCdk.GraplCdkStack(app, "MyTestStack", {
        stackName: STACK_NAME,
        version: "latest",
        env: ENV,
        operationalAlarmsEmail: "fake@fake.domain",
        securityAlarmsEmail: "fake@fake.domain",
        logLevels: {
            defaultLogLevel: "DEBUG",
            sysmonSubgraphGeneratorLogLevel: "DEBUG",
            osquerySubgraphGeneratorLogLevel: "DEBUG",
            nodeIdentifierLogLevel: "DEBUG",
            graphMergerLogLevel: "DEBUG",
            analyzerDispatcherLogLevel: "DEBUG",
            analyzerExecutorLogLevel: "DEBUG",
            engagementCreatorLogLevel: "DEBUG",
        },
    });

    const allConstructs = new CollectAllConstructs();
    cdk.Aspects.of(app).add(allConstructs);
    SynthUtils.synthesize(stack);

    test("fyi you can't test alarms", () => {
        // they just don't show up in the stack, it's a CDK issue
    });

    test("All dashboards have the deployment_name", () => {
        const constructs = allConstructs.getAllOfType(cloudwatch.Dashboard);

        for (const c of constructs) {
            const name = (c as any).physicalName;
            expect(name).toMatch(STARTS_WITH_STACK_NAME);
        }
    });
});

describe("Deployment names", () => {
    describe("Valid names", () => {
        const validTestCases = [
            "a123",
            "abc123",
            "a_a123",
            "a-123",
            "a",
            "b",
            "ab",
        ];
        for (const testCase of validTestCases) {
            test(`${testCase} works regardless of allowLegacy`, () => {
                validateDeploymentName(testCase, false);
                validateDeploymentName(testCase, true);
            });
        }
    });

    describe("Invalid names", () => {
        const invalidTestCases = [
            "testCases",
            "hello@",
            "Grapl-Test-Mar3",
            "1",
            "1a",
            "a1-_b",
            "123abc",
            "-abc",
            "_abc1",
            "abc--asd",
            "abc-_asd",
        ];
        for (const testCase of invalidTestCases) {
            describe(`case ${testCase}`, () => {
                test("Should fail when allow is false", () => {
                    expect(() =>
                        validateDeploymentName(testCase, false)
                    ).toThrowError(/is invalid/);
                });

                test("Should not fail when allow is true", () => {
                    validateDeploymentName(testCase, true);
                });
            });
        }
    });
});
