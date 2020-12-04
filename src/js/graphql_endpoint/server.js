const express = require('express');
const regexEscape = require('regex-escape');
const graphqlHTTP = require('express-graphql');
const schema = require('./modules/schema.js');
const cors = require('cors');
const app = express();
const awsServerlessExpress = require('aws-serverless-express')
const {validateJwt} = require('./modules/jwt.js');

console.log('server.js entrypoint')

const PORT = process.env.PORT || 5000;
const IS_LOCAL = (process.env.IS_LOCAL == 'True') || null;  // get this from environment

let origin = true;
let prefix = 'local-grapl';

if (!IS_LOCAL) {
    prefix = process.env.BUCKET_PREFIX;
    origin = process.env.UX_BUCKET_URL;
    console.log("origin: " + origin);
}

const corsRegexp = new RegExp(
    `https:\/\/${regexEscape(prefix)}-engagement-ux-bucket[.]s3([.][a-z]{2}-[a-z]{1,9}-\\d)?[.]amazonaws[.]com\/?`,
    'i'
);

console.log("corsRegexp", corsRegexp);

const corsDelegate = (req, callback) => {
    let corsOptions = {
        allowedHeaders: "Content-Type, Cookie, Access-Control-Allow-Headers, Authorization, X-Requested-With",
        credentials: true,
    };

    if(IS_LOCAL){
        console.log("Running Locally, CORS disabled")
        corsOptions = {...corsOptions, origin: true}
        callback(null, corsOptions);
        return; 
    }
    if (req.header('Origin') === origin) {
        console.log("exact matched origin: ", req.header('Origin'));
        corsOptions = {...corsOptions, origin: true}
    } else if (corsRegexp.test(req.header('Origin'))) {
        console.log("regexp matched origin: ", req.header('Origin'));
        corsOptions = {...corsOptions, origin: true}
    } else {
        console.log("invalid origin: ", req.header('Origin'));
        corsOptions = {...corsOptions, origin: false}
    }
    callback(null, corsOptions) // callback expects two parameters: error and options
}

const middleware = [cors(corsDelegate), validateJwt];

app.options('*', cors(corsDelegate));

if (IS_LOCAL) {
    app.use('/graphQlEndpoint/graphql', middleware, graphqlHTTP({
        schema: schema,
        graphiql: true
    }));
} else {
    app.use('/graphQlEndpoint/{proxy+}', middleware, graphqlHTTP({
        schema: schema,
        graphiql: false
    }));
    
}

app.use(function(req, res){
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
    exports.handler = (event, context) => {
        awsServerlessExpress.proxy(server, event, context)
    }
}
