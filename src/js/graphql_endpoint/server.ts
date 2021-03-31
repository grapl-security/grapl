import * as lambda from "aws-lambda";
import cors = require("cors");
import express = require("express");
import graphqlHTTP = require("express-graphql");
import { getRootQuerySchema } from "./modules/root_query.js";
import awsServerlessExpress = require("aws-serverless-express");
import { validateJwt } from "./modules/jwt.js";
//@ts-ignore
import regexEscape = require("regex-escape");

const app = express();
const PORT = process.env.PORT || 5000;
const IS_LOCAL = process.env.IS_LOCAL == "True" || null; // get this from environment

let origin: string | boolean = true;
let prefix = "local-grapl";

if (!IS_LOCAL) {
  prefix = process.env.DEPLOYMENT_NAME;
  origin = process.env.UX_BUCKET_URL;
  console.debug("origin: " + origin);
}

const corsRegexp = new RegExp(
  `https:\/\/${regexEscape(
    prefix
  )}-engagement-ux-bucket[.]s3([.][a-z]{2}-[a-z]{1,9}-\\d)?[.]amazonaws[.]com\/?`,
  "i"
);

console.debug("corsRegexp", corsRegexp);

type CorsCallback = (err: Error | null, options?: cors.CorsOptions) => void;

const corsDelegate = (req: cors.CorsRequest, callback: CorsCallback): void => {
  let corsOptions: cors.CorsOptions = {
    allowedHeaders:
      "Content-Type, Cookie, Access-Control-Allow-Headers, Authorization, X-Requested-With",
    credentials: true,
  };

  if (IS_LOCAL) {
    console.debug("Running Locally, CORS disabled");
    corsOptions = { ...corsOptions, origin: true };
    callback(null, corsOptions);
    return;
  }

  const originHeader = req.headers.origin;

  if (originHeader === origin) {
    console.debug("exact matched origin: ", originHeader);
    corsOptions = { ...corsOptions, origin: true };
  } else if (corsRegexp.test(originHeader)) {
    console.debug("regexp matched origin: ", originHeader);
    corsOptions = { ...corsOptions, origin: true };
  } else {
    console.debug("invalid origin: ", originHeader);
    corsOptions = { ...corsOptions, origin: false };
  }
  callback(null, corsOptions); // callback expects two parameters: error and options
};

const middleware = [cors(corsDelegate), validateJwt];

app.options("*", cors(corsDelegate));

app.use(
  "/graphQlEndpoint/graphql",
  middleware,
  graphqlHTTP(async (request, response, graphQLParams) => {
    console.debug({
      request: request,
      response: response,
      graphQLParams: graphQLParams,
    });
    return {
      schema: getRootQuerySchema(),
      graphiql: IS_LOCAL,
    };
  })
);

app.use(function (req, res) {
  console.warn(req);
  console.warn(req.path);
  res.sendStatus(404);
});

if (IS_LOCAL) {
  app.listen(PORT, function () {
    console.log("GraphQL Server started on Port " + PORT);
  });
} else {
  const server = awsServerlessExpress.createServer(app);
  console.log("AWS Server", server);
  exports.handler = (
    event: lambda.APIGatewayProxyEvent,
    context: lambda.Context
  ) => {
    awsServerlessExpress.proxy(server, event, context);
  };
}
