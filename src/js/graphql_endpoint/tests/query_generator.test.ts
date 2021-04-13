import { GraphQLInt, GraphQLString, GraphQLList } from "graphql";
import { allSchemasToGraphql } from "../modules/schema_to_graphql";
import { QueryGenerator } from "../modules/query_generator";
import { schemas } from "./schemas_fixture";

describe("QueryGenerator", () => {
    const GraplEntityType = allSchemasToGraphql(schemas);
    const queryGen = new QueryGenerator(GraplEntityType);

    describe("generate", () => {
        const generated = queryGen.generate();
        expect(generated).toEqual("hey");
    });
});
