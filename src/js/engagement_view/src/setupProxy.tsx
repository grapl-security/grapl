import { createProxyMiddleware } = require('http-proxy-middleware');
// TODO: THIS NEEDS TO BE REWORKED
module.exports = function(app: any) {
    app.use(
        '/api',
        createProxyMiddleware({
            target: '"http://127.0.0.1:1234"',
            changeOrigin: true,
        })
    );
};