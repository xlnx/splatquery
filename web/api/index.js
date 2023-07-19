import {createProxyMiddleware} from 'http-proxy-middleware';

const apiProxy = createProxyMiddleware({
  target: "https://api.1.koishi.top:80",
  changeOrigin: true,
  pathRewrite: {
    "^/api": "",
  }
});

export default async function handler(req, res) {
  return apiProxy(req, res);
}
