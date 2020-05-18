const express = require('express');
const graphqlHTTP = require('express-graphql');
const schema = require('./modules/schema.js');
const cors = require('cors');
const app = express();
const awsServerlessExpress = require('aws-serverless-express')
const { validateJwt } = require('./modules/jwt.js');

const PORT = process.env.PORT || 5000;
const IS_LOCAL = (process.env.IS_LOCAL == 'True') || null;  // get this from environment


// TODO: Move cors to its own module
const corsOptions = {
    origin: true,
    allowedHeaders: "Content-Type, Cookie, Access-Control-Allow-Headers, Authorization, X-Requested-With",
    credentials: true
};

const middleware = [cors(corsOptions)];

if (!IS_LOCAL) {
    middleware.push(validateJwt)
} else {
    console.info("Running locally - disabling auth");
}

app.use('/graphql', middleware, graphqlHTTP({
    schema: schema,
    graphiql: IS_LOCAL !== null
}));


if (IS_LOCAL) {
    app.listen(PORT, function () {
        console.log("GraphQL Server started on Port " + PORT);
    });
} else {
    const server = awsServerlessExpress.createServer(app)
    exports.handler = (event, context) => {
        awsServerlessExpress.proxy(server, event, context)
    }

}
