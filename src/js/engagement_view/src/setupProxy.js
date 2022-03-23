// NOTE: Despite this being Typescript project, this file must be JavaScript
// and named `setupProxy.js`. Otherwise it won't be picked up by `yarn start`,
// which is used for frontend development.
//
// The routes here need to be in sync with and point to backend APIs. Sites
// using React typically put all backend API requests behind `/api`, and we
// should probably do the same, so we don't have to worry about this file
// getting out of sync.
//
// The proxy `target` points to where the grapl-web-ui can be found when
// developing Grapl locally. We may want to parameterize this, making it easier
// to point to AWS deployments. This could be useful for:
//
//   1. Relieving a frontend developer's machine of resources. When making
//   changes to the frontend alone, a deployment Grapl being local doesn't
//   necessarily assist in the process. Local grapl consumes a fair amount of
//   system resources, and the development experience would probably be more
//   responsive by pointing to an AWS sandbox deloyment instead of running
//   locally. It would also improves flexibility of the development machine,
//   such as allowing the developer to restart the machine without redeploying.
//
//   2. Debugging graph rendering issues with particular data sets found in a
//   deployment.
//
// You should also be able to point it to a remote
// deployment as well, such as in AWS. This potentially be useful for something
// like debugging graph rendering issues seen in a deployment.
//
// https://create-react-app.dev/docs/proxying-api-requests-in-development/#configuring-the-proxy-manually

const { createProxyMiddleware } = require('http-proxy-middleware');

// TODO(inickles): Consider moving all endpoints behind `/api`.
module.exports = function(app) {
  app.use(
    '/auth',
    createProxyMiddleware({
      target: 'http://localhost:1234',
      changeOrigin: true,
    })
  );
  app.use(
    '/graphQlEndpoint',
    createProxyMiddleware({
      target: 'http://localhost:1234',
      changeOrigin: true,
    })
  );
};

// DEBUGGING: If you're here to debug something and are not very familiar with
// React, know that only requests from the React `fetch` API will be routed to
// the backend. You cannot, for example, use this middleware to forward a
// request from the browser URL or cmdline `curl` to localhost:3000. It must be
// a from frontend element using `fetch`. If desired, you could add a simple
// check to an App() definition to trigger requests for testing the middleware:
//
// ```
// function App() {
//   const ping = async () => {
//     const resp = await fetch('/api/ping');
//     console.log("PING", resp.body);
//   };
//
//   ping();
// ...
// ```
