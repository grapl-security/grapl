import { allSchemasToGraphql } from "../modules/schema_to_graphql";
import { QueryGenerator } from "../modules/query_generator";
import { schemas_fixture } from "./schemas_fixture";

describe("QueryGenerator", () => {
    const GraplEntityType = allSchemasToGraphql(schemas_fixture);
    const Risk = GraplEntityType.getTypes().find((t) => t.name == "Risk");
    const Asset = GraplEntityType.getTypes().find((t) => t.name == "Asset");
    const queryGen = new QueryGenerator(GraplEntityType);

    describe("generate", () => {
        it("smoke test - doesnt crash", () => {
            const generated = queryGen.generate();
            // Need to debug the generated query? Uncomment:
            // console.log(generated);
        });
    });

    describe("genOnFragment", () => {
        const expectedRisk = `... on Risk {
    uid,
    node_key,
    dgraph_type,
    display,
    analyzer_name,
    risk_score
}`;
        it("generates Risk correctly", () => {
            expect(
                queryGen.genOnFragment({ type: Risk, numSpaces: 0 })
            ).toEqual(expectedRisk);
        });

        it("prepends spaces", () => {
            const expectedRiskWithSpaces = expectedRisk
                .split("\n")
                .map((s) => `    ${s}`)
                .join("\n");
            expect(
                queryGen.genOnFragment({ type: Risk, numSpaces: 4 })
            ).toEqual(expectedRiskWithSpaces);
        });
    });

    describe("genField", () => {
        it("Handles scalars right", () => {
            const uidField = queryGen.genField({
                field: Asset.getFields()["uid"],
                stackLimit: 2,
                numSpaces: 0,
            });
            expect(uidField).toEqual(["uid"]);
        });
        it("Handles lists of scalars right", () => {
            const dgraphTypeField = queryGen.genField({
                field: Asset.getFields()["dgraph_type"],
                stackLimit: 2,
                numSpaces: 0,
            });
            expect(dgraphTypeField).toEqual(["dgraph_type"]);
        });
        it("Skips object when we're at the stack limit", () => {
            const risksField = queryGen.genField({
                field: Asset.getFields()["risks"],
                stackLimit: 0,
                numSpaces: 0,
            });
            expect(risksField).toEqual([]);
        });
        it("Handles objects when there's a stackLimit", () => {
            const risksField = queryGen.genField({
                field: Asset.getFields()["risks"],
                stackLimit: 2,
                numSpaces: 0,
            });
            expect(risksField).toEqual([
                `risks {
    uid,
    node_key,
    dgraph_type,
    display,
    analyzer_name,
    risk_score
}`,
            ]);
        });
    });
});
