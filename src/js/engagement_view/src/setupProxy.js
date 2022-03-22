const { createProxyMiddleware } = require('http-proxy-middleware');

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
