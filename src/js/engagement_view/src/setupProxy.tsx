const { createProxyMiddleware } = require('http-proxy-middleware');

module.exports = function(app) {
  app.use(
    '/',
    createProxyMiddleware({
      target: 'http://127.0.0.1:1234',
      changeOrigin: true,
    })
  );
};