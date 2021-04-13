import { GraphQLInt, GraphQLString, GraphQLList } from "graphql";
import { allSchemasToGraphql } from "../modules/schema_to_graphql";
import { schemas } from "./schemas_fixture";

describe("allSchemasToGraphql", () => {
    const GraplEntityType = allSchemasToGraphql(schemas);
    const subtypes = GraplEntityType.getTypes();

    describe("created a type Asset", () => {
        const AssetType = subtypes.find((t) => t.name == "Asset");
        const fields = AssetType.getFields();

        it("with a uid: int field", () => {
            expect(fields["uid"].type).toEqual(GraphQLInt);
        });

        it("with a display: string field", () => {
            expect(fields["display"].type).toEqual(GraphQLString);
        });

        it("with a dgraph_type: [string] field", () => {
            expect(fields["dgraph_type"].type).toEqual(
                GraphQLList(GraphQLString)
            );
        });
    });
});
