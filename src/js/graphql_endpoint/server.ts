import * as Tracing from "./tracing";
import * as express from "express";
import * as graphqlHTTP from "express-graphql";
import { getRootQuerySchema } from "./modules/root_query.js";
import { GraphQLError } from "graphql";

const app = express();
const PORT = process.env.PORT || 5000;
const IS_LOCAL = process.env.IS_LOCAL == "True" || null; // get this from environment

function customFormatErrorFnForDebugging(error: GraphQLError) {
    return {
        message: error.message,
        locations: error.locations,
        path: error.path,
    };
}

app.use(
    "/graphQlEndpoint/graphql",
    [],
    graphqlHTTP(async (request, response, graphQLParams) => {
        console.debug({
            graphQLParams: graphQLParams,
        });
        let schema;
        try {
            schema = await getRootQuerySchema();
        } catch (e) {
            console.error("Some uncaught promise error", e);
            throw e;
        }
        return {
            schema: schema,
            graphiql: IS_LOCAL,
            customFormatErrorFn: customFormatErrorFnForDebugging,
        };
    })
);

// Fallthrough if no route is found.
app.use(function (req, res) {
    console.warn(req.path);
    res.sendStatus(404);
});

app.listen(PORT, function () {
    console.log("GraphQL Server started on Port " + PORT);
});
